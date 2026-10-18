import AppKit
import Sparkle

/// 使用 Sparkle 验签和安装；后台仅通知应用显示新版标记。
final class AppUpdater: NSObject, SPUUpdaterDelegate, SPUStandardUserDriverDelegate {
    private var controller: SPUStandardUpdaterController!
    private var observations: [NSKeyValueObservation] = []
    private var started = false
    private let onChange: (() -> Void)?
    private(set) var availableVersion: String?
    private(set) var statusText: String?

    init(onChange: (() -> Void)? = nil) {
        self.onChange = onChange
        super.init()
        controller = SPUStandardUpdaterController(startingUpdater: false, updaterDelegate: self, userDriverDelegate: self)
        observations = [
            controller.updater.observe(\.canCheckForUpdates, options: [.new]) { [weak self] _, _ in self?.notifyChange() },
            controller.updater.observe(\.automaticallyChecksForUpdates, options: [.new]) { [weak self] _, _ in self?.notifyChange() }
        ]
    }

    var canCheckForUpdates: Bool { started && controller.updater.canCheckForUpdates }
    var automaticChecksEnabled: Bool {
        get { controller.updater.automaticallyChecksForUpdates }
        set { controller.updater.automaticallyChecksForUpdates = newValue }
    }

    func start() {
        guard !started else { return }
        // 预览与缺少签名配置的构建不启动网络检查。
        guard Bundle.main.object(forInfoDictionaryKey: "QuotaPreviewFile") == nil,
              let feed = Bundle.main.object(forInfoDictionaryKey: "SUFeedURL") as? String, !feed.isEmpty,
              let key = Bundle.main.object(forInfoDictionaryKey: "SUPublicEDKey") as? String, !key.isEmpty else {
            statusText = "此构建未启用在线更新"
            notifyChange()
            return
        }
        do {
            try controller.updater.start()
            started = true
            statusText = nil
        } catch {
            statusText = "更新服务暂不可用"
        }
        notifyChange()
    }

    func checkForUpdates() {
        guard canCheckForUpdates else { return }
        statusText = nil
        controller.checkForUpdates(nil)
        notifyChange()
    }

    var supportsGentleScheduledUpdateReminders: Bool { true }

    func standardUserDriverShouldHandleShowingScheduledUpdate(_ update: SUAppcastItem, andInImmediateFocus immediateFocus: Bool) -> Bool {
        // 包括启动时在内，后台发现新版都只显示应用自身的轻提示。
        false
    }

    func standardUserDriverWillHandleShowingUpdate(_ handleShowingUpdate: Bool, forUpdate update: SUAppcastItem, state: SPUUserUpdateState) {
        availableVersion = update.displayVersionString
        statusText = nil
        notifyChange()
    }

    func standardUserDriverWillFinishUpdateSession() {
        availableVersion = nil
        notifyChange()
    }

    func updaterDidNotFindUpdate(_ updater: SPUUpdater) {
        availableVersion = nil
        statusText = nil
        notifyChange()
    }

    func updater(_ updater: SPUUpdater, didAbortWithError error: Error) {
        // 错误详情交给 Sparkle 的手动检查窗口，不在后台弹窗或输出下载地址。
        let failure = error as NSError
        if failure.domain == SUSparkleErrorDomain && [Int(SUError.noUpdateError.rawValue), Int(SUError.installationCanceledError.rawValue)].contains(failure.code) {
            statusText = nil
        } else {
            statusText = "检查更新暂不可用"
        }
        notifyChange()
    }

    private func notifyChange() {
        if Thread.isMainThread { onChange?() }
        else { DispatchQueue.main.async { [weak self] in self?.onChange?() } }
    }
}
