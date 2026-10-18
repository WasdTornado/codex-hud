import AppKit

/// 隔离测试专用：记录真实更新事件，由界面操作确认安装。
final class UpdateFixtureDelegate: NSObject, NSApplicationDelegate {
    private var updater: AppUpdater!
    private var discovered = false
    private var lastState = ""
    private var window: NSWindow?
    private var updateButton: NSButton?
    private let logURL = URL(fileURLWithPath: Bundle.main.object(forInfoDictionaryKey: "FixtureLog") as! String)

    func record(_ event: String, _ details: [String: Any] = [:]) {
        var entry = details
        entry["event"] = event
        entry["time"] = Date().timeIntervalSince1970
        entry["build"] = Bundle.main.object(forInfoDictionaryKey: "CFBundleVersion") as? String
        guard let data = try? JSONSerialization.data(withJSONObject: entry), let handle = try? FileHandle(forWritingTo: logURL) else { return }
        defer { try? handle.close() }
        handle.seekToEndOfFile()
        handle.write(data)
        handle.write(Data("\n".utf8))
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(.accessory)
        record("launched", ["path": Bundle.main.bundlePath])
        if Bundle.main.object(forInfoDictionaryKey: "CFBundleVersion") as? String == "2" {
            DispatchQueue.main.asyncAfter(deadline: .now() + 1) { NSApp.terminate(nil) }
            return
        }
        updater = AppUpdater { [weak self] in self?.stateChanged() }
        updater.start()
    }

    private func stateChanged() {
        let state = "\(updater.availableVersion ?? "none")|\(updater.canCheckForUpdates)|\(updater.statusText ?? "")"
        if state != lastState {
            lastState = state
            record("state", ["state": state])
        }
        updateButton?.isEnabled = updater.canCheckForUpdates
        if updater.availableVersion != nil && !discovered {
            discovered = true
            record("gentle_update_found", ["version": updater.availableVersion!, "visibleWindows": NSApp.windows.filter(\.isVisible).count])
            // 记录后台没有弹窗后，创建隔离测试的操作入口。
            let panel = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 400, height: 150), styleMask: [.titled, .closable], backing: .buffered, defer: false)
            panel.title = "Codex HUD Update Fixture"
            let label = NSTextField(labelWithString: "已发现隔离测试新版 0.0.2")
            label.frame = NSRect(x: 24, y: 90, width: 350, height: 24)
            panel.contentView?.addSubview(label)
            let button = NSButton(title: "更新至 0.0.2…", target: self, action: #selector(checkForUpdates))
            button.frame = NSRect(x: 24, y: 28, width: 190, height: 36)
            button.isEnabled = updater.canCheckForUpdates
            panel.contentView?.addSubview(button)
            updateButton = button
            window = panel
            panel.center()
            panel.makeKeyAndOrderFront(nil)
        }
    }

    @objc private func checkForUpdates() {
        record("manual_check", ["canCheck": updater.canCheckForUpdates])
        updater.checkForUpdates()
    }
}

@main enum UpdateFixtureMain {
    static func main() {
        let delegate = UpdateFixtureDelegate()
        NSApplication.shared.delegate = delegate
        withExtendedLifetime(delegate) { NSApplication.shared.run() }
    }
}
