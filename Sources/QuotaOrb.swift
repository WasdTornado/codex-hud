import AppKit
import QuartzCore

final class FloatingPanel: NSPanel {
    var dismiss: (() -> Void)?
    var acceptsKeyboardFocus = true
    override var canBecomeKey: Bool { acceptsKeyboardFocus }
    override var canBecomeMain: Bool { false }
    override func keyDown(with event: NSEvent) {
        if event.keyCode == 53 { dismiss?() } else { super.keyDown(with: event) }
    }
}

class AppearanceView: NSView {
    var refreshColors: (() -> Void)?
    override func viewDidChangeEffectiveAppearance() {
        super.viewDidChangeEffectiveAppearance()
        effectiveAppearance.performAsCurrentDrawingAppearance { refreshColors?() }
        needsDisplay = true
    }
}

final class HUDGlassView: NSGlassEffectView {
    var prefersClearMaterial = false
    func followSystemAppearance(_ appearance: NSAppearance) {
        self.appearance = appearance
        contentView?.appearance = appearance
        // 这里只用于浅色；深色不创建玻璃视图，避免系统焦点与背景改变底色。
        style = prefersClearMaterial ? .clear : .regular
        tintColor = nil
    }
}

final class OrbInteractionView: AppearanceView {
    var clicked: (() -> Void)?
    var dragEnded: (() -> Void)?
    var pointerOrigin = NSPoint.zero
    var windowOrigin = NSPoint.zero
    var dragging = false
    override func acceptsFirstMouse(for event: NSEvent?) -> Bool { true }
    override func hitTest(_ point: NSPoint) -> NSView? {
        let rect = bounds.insetBy(dx: 8, dy: 8)
        return NSBezierPath(roundedRect: rect, xRadius: rect.width / 2, yRadius: rect.width / 2).contains(point) ? self : nil
    }
    override func mouseDown(with event: NSEvent) {
        guard let window else { return }
        pointerOrigin = NSEvent.mouseLocation
        windowOrigin = window.frame.origin
        dragging = false
    }
    override func mouseDragged(with event: NSEvent) {
        let pointer = NSEvent.mouseLocation
        let dx = pointer.x - pointerOrigin.x, dy = pointer.y - pointerOrigin.y
        if hypot(dx, dy) > 3 { dragging = true }
        if dragging { window?.setFrameOrigin(NSPoint(x: windowOrigin.x + dx, y: windowOrigin.y + dy)) }
    }
    override func mouseUp(with event: NSEvent) {
        if !dragging && bounds.contains(convert(event.locationInWindow, from: nil)) { clicked?() }
        dragEnded?()
    }
    override func accessibilityPerformPress() -> Bool { clicked?(); return true }
    override var acceptsFirstResponder: Bool { true }
    override func keyDown(with event: NSEvent) {
        if event.keyCode == 49 || event.keyCode == 36 { clicked?() } else { super.keyDown(with: event) }
    }
}

final class ProgressView: NSView {
    var value: Double? { didSet { needsDisplay = true } }
    var ring = false
    var capsuleHalf: Int?
    var color: NSColor { (value ?? 100) <= 10 ? .systemRed : (value ?? 100) <= 25 ? .systemOrange : .systemGreen }
    // 每个周期独占胶囊半周；按真实路径长度推进，直线与圆弧交界处保持连续。
    private func capsulePath(fraction: CGFloat, lower: Bool) -> NSBezierPath {
        let inset: CGFloat = 2.5
        let radius = bounds.width / 2 - inset
        let center = NSPoint(x: bounds.midX, y: bounds.maxY - inset - radius)
        let startY = bounds.midY + 2
        let straight = center.y - startY
        let arcLength = CGFloat.pi * radius
        var distance = max(0, min(1, fraction)) * (straight * 2 + arcLength)
        let path = NSBezierPath()
        path.move(to: NSPoint(x: inset, y: startY))
        path.line(to: NSPoint(x: inset, y: startY + min(straight, distance)))
        distance -= straight
        if distance > 0 {
            let angle = min(arcLength, distance) / radius * 180 / .pi
            path.appendArc(withCenter: center, radius: radius, startAngle: 180, endAngle: 180 - angle, clockwise: true)
            distance -= arcLength
            if distance > 0 {
                path.line(to: NSPoint(x: bounds.maxX - inset, y: center.y - min(straight, distance)))
            }
        }
        if lower {
            var transform = AffineTransform()
            transform.translate(x: bounds.width, y: bounds.height)
            transform.rotate(byDegrees: 180)
            path.transform(using: transform)
        }
        path.lineCapStyle = .round
        return path
    }
    override func draw(_ dirtyRect: NSRect) {
        if let capsuleHalf {
            let track = capsulePath(fraction: 1, lower: capsuleHalf == 1)
            track.lineWidth = 1.4
            NSColor.labelColor.withAlphaComponent(0.06).setStroke(); track.stroke()
            if let value, value > 0 {
                let progress = capsulePath(fraction: CGFloat(value / 100), lower: capsuleHalf == 1)
                progress.lineWidth = 1.6
                color.withAlphaComponent(0.8).setStroke(); progress.stroke()
            }
        } else if ring {
            let center = NSPoint(x: bounds.midX, y: bounds.midY)
            let radius = bounds.width / 2 - 1
            let track = NSBezierPath(ovalIn: bounds.insetBy(dx: 1, dy: 1))
            track.lineWidth = 1.4
            NSColor.labelColor.withAlphaComponent(0.06).setStroke(); track.stroke()
            if let value, value > 0 {
                let path = NSBezierPath()
                path.appendArc(withCenter: center, radius: radius, startAngle: 90, endAngle: 90 - CGFloat(min(100, value)) * 3.6, clockwise: true)
                path.lineWidth = 1.6; path.lineCapStyle = .round
                color.withAlphaComponent(0.8).setStroke(); path.stroke()
            }
        } else {
            NSColor.labelColor.withAlphaComponent(0.10).setFill()
            NSBezierPath(roundedRect: bounds, xRadius: bounds.height / 2, yRadius: bounds.height / 2).fill()
            if let value, value > 0 {
                color.withAlphaComponent(0.82).setFill()
                NSBezierPath(roundedRect: NSRect(x: 0, y: 0, width: bounds.width * CGFloat(min(100, value) / 100), height: bounds.height), xRadius: bounds.height / 2, yRadius: bounds.height / 2).fill()
            }
        }
    }
}

@main
final class QuotaOrb: NSObject, NSApplicationDelegate, NSWindowDelegate {
    var orb: FloatingPanel!
    var details: FloatingPanel!
    var orbSurface: OrbInteractionView!
    var orbNumbers: [NSTextField] = []
    var orbProgress: [ProgressView] = []
    var detailNumbers: [NSTextField] = []
    var detailResets: [NSTextField] = []
    var detailProgress: [ProgressView] = []
    var syncLabel: NSTextField!
    var statusDot: NSView!
    var refreshButton: NSButton!
    var statusItem: NSStatusItem!
    let menuBarStatus = MenuBarStatus()
    var updateMenuItem: NSMenuItem!
    var automaticUpdatesMenuItem: NSMenuItem!
    var desktopHUDMenuItem: NSMenuItem!
    var desktopHUDVisible = true
    var appUpdater: AppUpdater?
    var quota = QuotaSnapshot.empty
    var layoutKey = ""
    let client = QuotaClient()
    lazy var store = QuotaStore { [weak self] completion in
        guard let self else { return }
        DispatchQueue.global(qos: .utility).async {
            completion(Result { try self.client.read() })
        }
    }
    var appearanceObservation: NSKeyValueObservation?
    var appliedDarkAppearance: Bool?
    var refreshTimer: Timer?
    var outsideClickMonitor: Any?
    var localClickMonitor: Any?
    var clock: Timer?
    var motionTimer: Timer?
    var motionTime = CACurrentMediaTime()
    var reveal: Double = 0
    var velocity: Double = 0
    var expanded = false
    var changingFrame = false
    var previewFile: String?
    var previewData: Data?
    let defaults = UserDefaults.standard
    var windows: [QuotaWindow] { quota.windows.isEmpty ? [QuotaWindow()] : quota.windows }
    var dual: Bool { windows.count > 1 }

    static func main() {
        let app = NSApplication.shared
        let delegate = QuotaOrb()
        app.delegate = delegate
        app.setActivationPolicy(.accessory)
        app.run()
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        let args = CommandLine.arguments
        if let i = args.firstIndex(of: "--preview"), args.count > i + 1 { previewFile = args[i + 1] }
        if previewFile == nil { previewFile = Bundle.main.object(forInfoDictionaryKey: "QuotaPreviewFile") as? String }
        desktopHUDVisible = !defaults.bool(forKey: "desktopHUDHidden")
        orb = panel(size: NSSize(width: 58, height: 58), title: "Codex HUD · 额度小球")
        // 小球只处理点击和拖动，键盘焦点交给展开后的详情。
        orb.acceptsKeyboardFocus = false
        details = panel(size: NSSize(width: 208, height: 110), title: "Codex HUD · 额度详情")
        orb.delegate = self
        orb.dismiss = { [weak self] in self?.setExpanded(false) }
        details.dismiss = { [weak self] in self?.setExpanded(false) }
        // 新应用只迁移旧版窗口位置，保留原项目与原偏好以便回退。
        if defaults.object(forKey: "x") == nil, let old = UserDefaults(suiteName: "com.wasdtornado.quotaorb"), old.object(forKey: "x") != nil {
            defaults.set(old.double(forKey: "x"), forKey: "x")
            defaults.set(old.double(forKey: "y"), forKey: "y")
        }
        makeMenu()
        if previewFile == nil {
            appUpdater = AppUpdater(onChange: { [weak self] in self?.renderUpdateState() })
            appUpdater?.start()
        }
        renderUpdateState()
        let area = NSScreen.main?.visibleFrame ?? NSRect(x: 0, y: 0, width: 1440, height: 900)
        let origin = previewFile == nil && defaults.object(forKey: "x") != nil
            ? NSPoint(x: defaults.double(forKey: "x"), y: defaults.double(forKey: "y"))
            : NSPoint(x: area.maxX - orb.frame.width - 8, y: area.midY)
        orb.setFrameOrigin(origin)
        constrainOrb()
        outsideClickMonitor = NSEvent.addGlobalMonitorForEvents(matching: [.leftMouseDown, .rightMouseDown]) { [weak self] _ in self?.dismissOutside() }
        localClickMonitor = NSEvent.addLocalMonitorForEvents(matching: [.leftMouseDown, .rightMouseDown]) { [weak self] event in self?.dismissOutside(); return event }
        if previewFile != nil { readPreview() } else {
            store.changed = { [weak self] state in self?.quota = state; self?.render() }
            store.refresh()
            refreshTimer = Timer.scheduledTimer(withTimeInterval: QuotaRefreshPolicy.interval, repeats: true) { [weak self] _ in self?.refresh() }
        }
        clock = Timer.scheduledTimer(withTimeInterval: 1, repeats: true) { [weak self] _ in
            self?.readPreview(); self?.render()
        }
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(woke), name: NSWorkspace.didWakeNotification, object: nil)
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(accessibilityChanged), name: NSWorkspace.accessibilityDisplayOptionsDidChangeNotification, object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(screensChanged), name: NSApplication.didChangeScreenParametersNotification, object: nil)
        appearanceObservation = NSApp.observe(\.effectiveAppearance, options: [.new]) { [weak self] _, _ in
            DispatchQueue.main.async { self?.applySystemAppearance() }
        }
        applySystemAppearance()
        if desktopHUDVisible { orb.orderFrontRegardless() }
    }

    func applySystemAppearance() {
        let name: NSAppearance.Name = NSApp.effectiveAppearance.bestMatch(from: [.aqua, .darkAqua]) == .darkAqua ? .darkAqua : .aqua
        guard let appearance = NSAppearance(named: name) else { return }
        let dark = name == .darkAqua
        let materialChanged = appliedDarkAppearance != dark
        appliedDarkAppearance = dark
        func update(_ view: NSView) {
            if let glass = view as? HUDGlassView { glass.followSystemAppearance(appearance) }
            if let adaptive = view as? AppearanceView {
                appearance.performAsCurrentDrawingAppearance { adaptive.refreshColors?() }
            }
            view.needsDisplay = true
            view.subviews.forEach(update)
        }
        for panel in [orb, details] {
            panel?.appearance = appearance
            if let content = panel?.contentView { update(content) }
        }
        // 深浅色使用不同材质，在切换时重新创建表面，彻底移除旧玻璃层。
        if materialChanged { layoutKey = ""; render() }
    }

    func panel(size: NSSize, title: String) -> FloatingPanel {
        let panel = FloatingPanel(contentRect: NSRect(origin: .zero, size: size), styleMask: [.borderless, .nonactivatingPanel], backing: .buffered, defer: false)
        panel.title = title
        panel.level = .floating
        panel.isOpaque = false; panel.backgroundColor = .clear; panel.hasShadow = false
        panel.hidesOnDeactivate = false; panel.isReleasedWhenClosed = false
        panel.isMovableByWindowBackground = false
        panel.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
        return panel
    }

    func glass(in root: NSView, size: NSSize, radius: CGFloat, clearMaterial: Bool = false) -> (NSView, NSGlassEffectView?) {
        let rect = NSRect(origin: NSPoint(x: 8, y: 8), size: size)
        let content = AppearanceView(frame: NSRect(origin: .zero, size: size))
        let dark = NSApp.effectiveAppearance.bestMatch(from: [.aqua, .darkAqua]) == .darkAqua
        if dark || NSWorkspace.shared.accessibilityDisplayShouldReduceTransparency {
            content.appearance = NSAppearance(named: dark ? .darkAqua : .aqua)
            content.frame = rect; content.wantsLayer = true
            content.clipsToBounds = true
            content.layer?.masksToBounds = true
            content.refreshColors = { [weak content] in
                // 深色完全不透明，按下、展开和失去焦点都使用同一底色。
                content?.layer?.backgroundColor = dark ? NSColor(calibratedWhite: 0.10, alpha: 1).cgColor : NSColor.windowBackgroundColor.cgColor
                content?.layer?.borderColor = dark ? NSColor.white.withAlphaComponent(0.10).cgColor : NSColor.separatorColor.cgColor
            }
            content.effectiveAppearance.performAsCurrentDrawingAppearance { content.refreshColors?() }
            content.layer?.cornerRadius = radius
            content.layer?.borderWidth = 0.5
            root.addSubview(content)
            return (content, nil)
        }
        // 只用系统玻璃；不再叠加深色遮罩或第二层模糊材质。
        // 系统玻璃自身也会绘制外缘阴影；通过父视图裁切限制到实际轮廓。
        let clipping = NSView(frame: rect)
        clipping.wantsLayer = true
        clipping.clipsToBounds = true
        clipping.layer?.backgroundColor = NSColor.clear.cgColor
        clipping.layer?.cornerRadius = radius
        clipping.layer?.masksToBounds = true
        let mask = CAShapeLayer()
        mask.path = CGPath(roundedRect: clipping.bounds, cornerWidth: radius, cornerHeight: radius, transform: nil)
        clipping.layer?.mask = mask
        root.addSubview(clipping)
        if clearMaterial {
            // 小球使用固定激活状态的原生磨砂材质，不参与 Liquid Glass 的点击响应。
            let frost = NSVisualEffectView(frame: NSRect(origin: .zero, size: size))
            frost.material = .popover
            frost.blendingMode = .behindWindow
            frost.state = .active
            frost.appearance = NSAppearance(named: .aqua)
            frost.addSubview(content)
            clipping.addSubview(frost)
            clipping.layer?.borderWidth = 0.5
            clipping.layer?.borderColor = NSColor.white.withAlphaComponent(0.55).cgColor
            return (content, nil)
        }
        let glass = HUDGlassView(frame: NSRect(origin: .zero, size: size))
        // 小球使用更通透的系统材质，详情保留常规材质以保证文字可读。
        glass.prefersClearMaterial = clearMaterial
        glass.style = clearMaterial ? .clear : .regular
        glass.cornerRadius = radius
        glass.contentView = content
        if #available(macOS 27.0, *) { glass.effectIsInteractive = false }
        clipping.addSubview(glass)
        glass.followSystemAppearance(NSAppearance(named: NSApp.effectiveAppearance.bestMatch(from: [.aqua, .darkAqua]) == .darkAqua ? .darkAqua : .aqua)!)
        return (content, glass)
    }

    func label(_ text: String, _ frame: NSRect, _ size: CGFloat, _ weight: NSFont.Weight = .regular, _ color: NSColor = .labelColor) -> NSTextField {
        let field = NSTextField(labelWithString: text)
        field.frame = frame; field.font = .systemFont(ofSize: size, weight: weight)
        field.textColor = color; field.isSelectable = false
        field.lineBreakMode = .byTruncatingTail; field.maximumNumberOfLines = 1
        return field
    }

    func rebuild() {
        let size = dual ? NSSize(width: 36, height: 72) : NSSize(width: 36, height: 36)
        let old = orb.frame
        changingFrame = true
        orb.setFrame(NSRect(x: old.maxX - size.width - 16, y: old.midY - (size.height + 16) / 2, width: size.width + 16, height: size.height + 16), display: true)
        changingFrame = false
        orbSurface = OrbInteractionView(frame: NSRect(origin: .zero, size: orb.frame.size))
        orbSurface.wantsLayer = true
        orbSurface.setAccessibilityElement(true); orbSurface.setAccessibilityRole(.button)
        let (content, _) = glass(in: orbSurface, size: size, radius: size.width / 2, clearMaterial: true)
        orbNumbers = []; orbProgress = []
        for (i, window) in windows.prefix(2).enumerated() {
            let valueY: CGFloat = dual ? (i == 0 ? 38 : 10) : 9
            let number = label("—", NSRect(x: 0, y: valueY, width: size.width, height: 20), 16, .semibold)
            number.alignment = .center; content.addSubview(number); orbNumbers.append(number)
            if dual {
                let name = label(window.shortName, NSRect(x: 0, y: i == 0 ? 56 : 28, width: size.width, height: 10), 8, .medium, .secondaryLabelColor)
                name.alignment = .center; content.addSubview(name)
            }
            let progress = ProgressView(frame: dual ? NSRect(origin: .zero, size: size) : NSRect(x: 1.5, y: 1.5, width: 33, height: 33))
            progress.ring = !dual
            progress.capsuleHalf = dual ? i : nil
            content.addSubview(progress, positioned: .below, relativeTo: number)
            orbProgress.append(progress)
        }
        statusDot = NSView(frame: NSRect(x: size.width / 2 - 1.5, y: 5, width: 3, height: 3))
        statusDot.wantsLayer = true; statusDot.layer?.cornerRadius = 1.5
        content.addSubview(statusDot)
        orb.contentView = orbSurface
        orbSurface.clicked = { [weak self] in self?.toggleDetails() }
        orbSurface.refreshColors = { [weak self] in self?.render() }
        orbSurface.dragEnded = { [weak self] in
            guard let self else { return }
            constrainOrb()
            if previewFile == nil { defaults.set(orb.frame.minX, forKey: "x"); defaults.set(orb.frame.minY, forKey: "y") }
            if details.isVisible { applyMotion() }
        }
        rebuildDetails()
        if old.origin != .zero { constrainOrb() }
    }

    func rebuildDetails() {
        let height = CGFloat(54 + windows.count * 39)
        details.setContentSize(NSSize(width: 208, height: height + 16))
        let root = NSView(frame: NSRect(origin: .zero, size: details.frame.size))
        let (content, _) = glass(in: root, size: NSSize(width: 192, height: height), radius: 18)
        content.addSubview(label(previewFile == nil ? "剩余额度" : "剩余额度 · 预览", NSRect(x: 13, y: height - 29, width: 142, height: 18), 11, .semibold))
        detailNumbers = []; detailResets = []; detailProgress = []
        for (i, window) in windows.enumerated() {
            let y = height - 48 - CGFloat(i * 39)
            content.addSubview(label(window.name, NSRect(x: 13, y: y, width: 105, height: 15), 10, .medium))
            let value = label("—", NSRect(x: 122, y: y - 1, width: 57, height: 17), 13, .semibold)
            value.alignment = .right; content.addSubview(value); detailNumbers.append(value)
            let progress = ProgressView(frame: NSRect(x: 13, y: y - 6, width: 166, height: 3))
            content.addSubview(progress); detailProgress.append(progress)
            let reset = label("", NSRect(x: 13, y: y - 21, width: 166, height: 12), 8.5, .medium, .secondaryLabelColor)
            content.addSubview(reset); detailResets.append(reset)
        }
        syncLabel = label("正在连接…", NSRect(x: 13, y: 7, width: 139, height: 13), 9, .medium)
        content.addSubview(syncLabel)
        refreshButton = NSButton(image: NSImage(systemSymbolName: "arrow.clockwise", accessibilityDescription: "刷新额度")!, target: self, action: #selector(refresh))
        refreshButton.frame = NSRect(x: 155, y: 0, width: 28, height: 26)
        refreshButton.isBordered = false; refreshButton.contentTintColor = .secondaryLabelColor
        refreshButton.toolTip = previewFile == nil ? "立即刷新 · 每 5 分钟自动同步" : "重新读取预览样本，不连接真实账号"
        content.addSubview(refreshButton)
        details.contentView = root
    }

    func constrainOrb() {
        guard !changingFrame else { return }
        let center = NSPoint(x: orb.frame.midX, y: orb.frame.midY)
        let area = NSScreen.screens.first(where: { $0.frame.contains(center) })?.visibleFrame ?? orb.screen?.visibleFrame ?? NSScreen.main!.visibleFrame
        var frame = orb.frame
        // 透明边距外仍留少量空隙，数字与玻璃永远不被屏幕边缘裁切。
        frame.origin.x = max(area.minX, min(frame.minX, area.maxX - frame.width))
        frame.origin.y = max(area.minY, min(frame.minY, area.maxY - frame.height))
        if frame != orb.frame { changingFrame = true; orb.setFrame(frame, display: true); changingFrame = false }
    }

    func detailFrame() -> NSRect {
        let area = orb.screen?.visibleFrame ?? NSScreen.main!.visibleFrame
        let size = details.frame.size
        let toRight = orb.frame.midX < area.midX
        let x = toRight ? orb.frame.maxX - 2 : orb.frame.minX - size.width + 2
        return NSRect(x: max(area.minX, min(x, area.maxX - size.width)), y: max(area.minY, min(orb.frame.midY - size.height / 2, area.maxY - size.height)), width: size.width, height: size.height)
    }

    func dismissOutside() {
        guard expanded else { return }
        let point = NSEvent.mouseLocation
        if !orb.frame.contains(point) && !details.frame.contains(point) { setExpanded(false) }
    }

    func setExpanded(_ value: Bool) {
        guard desktopHUDVisible || !value else { return }
        expanded = value
        if value {
            render()
            applyMotion()
            details.makeKeyAndOrderFront(nil)
        }
        motionTime = CACurrentMediaTime()
        if motionTimer == nil {
            let timer = Timer(timeInterval: 1 / 120, repeats: true) { [weak self] _ in self?.stepMotion() }
            motionTimer = timer; RunLoop.main.add(timer, forMode: .common)
        }
    }

    func stepMotion() {
        let now = CACurrentMediaTime(), dt = min(0.05, now - motionTime)
        motionTime = now
        let target = expanded ? 1.0 : 0.0
        // 临界阻尼弹簧的解析步进：反向点击沿当前进度与速度继续，无跳帧或输入锁。
        let omega = 28.0, delta = reveal - target, c = velocity + omega * delta, decay = exp(-omega * dt)
        reveal = target + (delta + c * dt) * decay
        velocity = (velocity - omega * c * dt) * decay
        if abs(reveal - target) < 0.001 && abs(velocity) < 0.02 {
            reveal = target; velocity = 0; motionTimer?.invalidate(); motionTimer = nil
        }
        applyMotion()
        if reveal == 0 && !expanded { details.orderOut(nil) }
    }

    func applyMotion() {
        guard details != nil, orb != nil else { return }
        var frame = detailFrame()
        if !NSWorkspace.shared.accessibilityDisplayShouldReduceMotion {
            let sign: CGFloat = frame.midX > orb.frame.midX ? -1 : 1
            frame.origin.x += sign * 10 * CGFloat(1 - reveal)
        }
        details.setFrame(frame, display: true)
        details.alphaValue = max(0, min(1, reveal))
    }

    func makeMenu() {
        let menu = NSMenu()
        menu.autoenablesItems = false
        menu.addItem(withTitle: previewFile == nil ? "Codex HUD · 额度小球" : "Codex HUD · 预览数据", action: nil, keyEquivalent: "").isEnabled = false
        menu.addItem(.separator())
        menu.addItem(withTitle: "刷新额度", action: #selector(refresh), keyEquivalent: "r").target = self
        desktopHUDMenuItem = menu.addItem(withTitle: "显示桌面悬浮窗", action: #selector(toggleDesktopHUD), keyEquivalent: "")
        desktopHUDMenuItem.target = self
        menu.addItem(withTitle: "显示 / 收起详情", action: #selector(toggleDetails), keyEquivalent: "i").target = self
        menu.addItem(withTitle: "靠左停靠", action: #selector(dockLeft), keyEquivalent: "[").target = self
        menu.addItem(withTitle: "靠右停靠", action: #selector(dockRight), keyEquivalent: "]").target = self
        menu.addItem(.separator())
        updateMenuItem = menu.addItem(withTitle: "检查更新…", action: #selector(checkAppUpdates), keyEquivalent: "")
        updateMenuItem.target = self
        automaticUpdatesMenuItem = menu.addItem(withTitle: "自动检查更新", action: #selector(toggleAutomaticUpdates), keyEquivalent: "")
        automaticUpdatesMenuItem.target = self
        menu.addItem(withTitle: "关于 Codex HUD", action: #selector(showAbout), keyEquivalent: "").target = self
        menu.addItem(withTitle: "退出", action: #selector(quit), keyEquivalent: "q").target = self
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        statusItem.menu = menu
        let appMenu = NSMenu(), root = NSMenuItem()
        root.submenu = menu.copy() as? NSMenu; appMenu.addItem(root); NSApplication.shared.mainMenu = appMenu
        renderDesktopHUDState()
    }

    func renderDesktopHUDState() {
        // 菜单栏与应用菜单各有一份菜单，同步勾选状态和依赖浮窗的操作。
        let menus = [statusItem.menu, NSApp.mainMenu?.items.first?.submenu]
        for menu in menus {
            for item in menu?.items ?? [] {
                if item.action == #selector(toggleDesktopHUD) {
                    item.state = desktopHUDVisible ? .on : .off
                    item.toolTip = "隐藏小球或胶囊后，菜单栏额度仍会正常刷新"
                } else if [#selector(toggleDetails), #selector(dockLeft), #selector(dockRight)].contains(item.action) {
                    item.isEnabled = desktopHUDVisible
                }
            }
        }
    }

    @objc func toggleDesktopHUD() {
        desktopHUDVisible.toggle()
        defaults.set(!desktopHUDVisible, forKey: "desktopHUDHidden")
        if desktopHUDVisible {
            constrainOrb()
            orb.orderFrontRegardless()
        } else {
            // 立即停止详情动画，避免计时器或残留详情让已隐藏的窗口重新出现。
            expanded = false; reveal = 0; velocity = 0
            motionTimer?.invalidate(); motionTimer = nil
            details.orderOut(nil)
            orb.orderOut(nil)
        }
        renderDesktopHUDState()
    }

    func render() {
        let key = windows.map { String($0.minutes ?? -1) }.joined(separator: ",")
        if layoutKey != key { layoutKey = key; rebuild() }
        let age = quota.updatedAt.map { max(0, Int(Date().timeIntervalSince1970 - $0 / 1000)) }
        let hasValue = quota.windows.contains { $0.remaining != nil }
        let warning = QuotaRefreshPolicy.isStale(updatedAt: quota.updatedAt, now: Date().timeIntervalSince1970, hasError: quota.error != nil)
        let elapsed = age.map { $0 < 60 ? "\($0) 秒前" : $0 < 3600 ? "\($0 / 60) 分钟前" : "\($0 / 3600) 小时前" } ?? "尚未同步"
        let message: String
        if quota.loading { message = "正在同步…" }
        else if quota.error != nil { message = hasValue ? "更新失败 · \(elapsed)" : "连接失败 · 点击重试" }
        else if !hasValue { message = "额度暂不可用" }
        else if warning { message = "待更新 · \(elapsed)" }
        else { message = (age ?? 0) < 5 ? "刚刚更新" : "\(elapsed)更新" }
        let statusColor: NSColor = warning ? .systemOrange : quota.loading || !hasValue ? .secondaryLabelColor : .systemGreen
        orbSurface.effectiveAppearance.performAsCurrentDrawingAppearance {
            statusDot.layer?.backgroundColor = statusColor.cgColor
        }
        syncLabel.stringValue = "●  " + message
        syncLabel.textColor = warning ? .systemOrange : .secondaryLabelColor
        syncLabel.toolTip = quota.updatedAt.map { stamp in
            let formatter = DateFormatter(); formatter.locale = Locale(identifier: "zh_CN"); formatter.dateFormat = "M月d日 HH:mm:ss"
            return "上次成功同步：" + formatter.string(from: Date(timeIntervalSince1970: stamp / 1000))
        }
        for (i, window) in windows.enumerated() {
            if i < orbNumbers.count {
                let digits = window.value.replacingOccurrences(of: "%", with: "")
                // 百分号仅在详情保留；三位数也放在同一个居中框中。
                orbNumbers[i].font = .monospacedDigitSystemFont(ofSize: digits.count >= 3 ? 13 : 16, weight: .semibold)
                orbNumbers[i].stringValue = digits
                orbProgress[i].value = window.remaining
            }
            detailNumbers[i].stringValue = window.value
            detailProgress[i].value = window.remaining
            detailResets[i].stringValue = resetText(window.resetsAt)
        }
        refreshButton.isEnabled = !quota.loading
        let summary = quota.windows.map { "\($0.name)剩余 \($0.value)" }.joined(separator: "，")
        orbSurface.setAccessibilityLabel((previewFile == nil ? "" : "预览数据，") + "Codex，" + (summary.isEmpty ? "正在读取额度" : summary) + "，\(message)，点击\(expanded ? "收起" : "查看")详情")
        menuBarStatus.update(statusItem.button, quota: quota, warning: warning, message: message, updateAvailable: appUpdater?.availableVersion != nil, preview: previewFile != nil)
        if details.isVisible && motionTimer == nil { applyMotion() }
    }

    func resetText(_ timestamp: Double?) -> String {
        guard let timestamp else { return "重置时间暂不可用" }
        let remaining = Int(timestamp - Date().timeIntervalSince1970)
        if remaining <= 0 { return "等待服务端更新重置时间" }
        if remaining < 3600 { return "\(max(1, remaining / 60)) 分钟后重置" }
        if remaining < 86400 { return "\(remaining / 3600) 小时 \((remaining % 3600) / 60) 分钟后重置" }
        let formatter = DateFormatter(); formatter.locale = Locale(identifier: "zh_CN"); formatter.dateFormat = "E HH:mm"
        return formatter.string(from: Date(timeIntervalSince1970: timestamp)) + " 重置"
    }

    func readPreview() {
        // 预览只能由显式启动参数启用，数据标识保留在详情、菜单和无障碍文本中。
        guard let previewFile, let data = try? Data(contentsOf: URL(fileURLWithPath: previewFile)), data != previewData,
              let state = try? JSONDecoder().decode(QuotaSnapshot.self, from: data) else { return }
        // 外观样本只作用于明确的预览构建，不修改系统设置或正式应用偏好。
        if let sample = try? JSONSerialization.jsonObject(with: data) as? [String: Any] {
            let theme = sample["previewAppearance"] as? String
            NSApp.appearance = theme == "dark" ? NSAppearance(named: .darkAqua) : theme == "light" ? NSAppearance(named: .aqua) : nil
        }
        previewData = data; quota = state; render(); applySystemAppearance()
    }

    @objc func refresh() {
        if previewFile != nil { previewData = nil; readPreview(); return }
        store.refresh()
    }
    func renderUpdateState() {
        guard updateMenuItem != nil else { return }
        updateMenuItem.title = appUpdater?.availableVersion.map { "更新至 \($0)…" } ?? "检查更新…"
        updateMenuItem.isEnabled = appUpdater?.canCheckForUpdates == true
        automaticUpdatesMenuItem.isEnabled = appUpdater != nil
        automaticUpdatesMenuItem.state = appUpdater?.automaticChecksEnabled == true ? .on : .off
        updateMenuItem.toolTip = previewFile == nil ? appUpdater?.statusText : "预览模式不检查应用更新"
        // 菜单栏菜单和应用菜单使用不同实例，更新状态需同步给两个入口。
        for item in NSApplication.shared.mainMenu?.items.first?.submenu?.items ?? [] {
            if item.action == #selector(checkAppUpdates) {
                item.title = updateMenuItem.title; item.isEnabled = updateMenuItem.isEnabled; item.toolTip = updateMenuItem.toolTip
            } else if item.action == #selector(toggleAutomaticUpdates) {
                item.state = automaticUpdatesMenuItem.state; item.isEnabled = automaticUpdatesMenuItem.isEnabled
            }
        }
        render()
    }
    @objc func checkAppUpdates() { appUpdater?.checkForUpdates() }
    @objc func toggleAutomaticUpdates() {
        guard let appUpdater else { return }
        appUpdater.automaticChecksEnabled.toggle()
        renderUpdateState()
    }
    @objc func accessibilityChanged() { layoutKey = ""; render() }
    func dock(right: Bool) {
        let area = orb.screen?.visibleFrame ?? NSScreen.main!.visibleFrame
        orb.setFrameOrigin(NSPoint(x: right ? area.maxX - orb.frame.width : area.minX, y: orb.frame.minY))
        orbSurface.dragEnded?()
    }
    @objc func dockLeft() { dock(right: false) }
    @objc func dockRight() { dock(right: true) }
    @objc func screensChanged() { constrainOrb(); if details.isVisible { applyMotion() } }
    @objc func woke() { refresh() }
    @objc func toggleDetails() { setExpanded(!expanded) }
    @objc func showAbout() {
        NSApplication.shared.activate(ignoringOtherApps: true)
        NSApplication.shared.orderFrontStandardAboutPanel(options: [
            .applicationName: "Codex HUD",
            .credits: NSAttributedString(string: "作者：wasdTornado\n小巧的 Codex 额度悬浮工具")
        ])
    }
    @objc func quit() { NSApplication.shared.terminate(nil) }
    func windowDidMove(_ notification: Notification) { if !changingFrame && details?.isVisible == true { applyMotion() } }
    func applicationWillTerminate(_ notification: Notification) {
        clock?.invalidate(); motionTimer?.invalidate()
        refreshTimer?.invalidate(); client.cancel()
        if let outsideClickMonitor { NSEvent.removeMonitor(outsideClickMonitor) }
        if let localClickMonitor { NSEvent.removeMonitor(localClickMonitor) }

    }
}
