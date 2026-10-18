import AppKit

/// 环内只显示一个读数，完整周期和更新状态通过提示与菜单提供。
final class MenuBarStatus {
    private var lastImageKey = ""

    func update(_ button: NSStatusBarButton?, quota: QuotaSnapshot, warning: Bool, message: String, updateAvailable: Bool, preview: Bool) {
        guard let button else { return }
        // 双周期固定优先 5h，不能把周额度的进度环和 5h 数字混在一起。
        let selected = quota.windows.first(where: { $0.minutes == 300 }) ?? quota.windows.first
        let remaining = selected?.remaining
        let digits = selected?.value.replacingOccurrences(of: "%", with: "") ?? "—"
        button.title = ""
        button.imagePosition = .imageOnly
        let key = "\(remaining ?? -1)-\(digits)-\(quota.loading)-\(warning)-\(updateAvailable)"
        if key != lastImageKey {
            lastImageKey = key
            button.image = gauge(remaining: remaining, digits: digits, loading: quota.loading, warning: warning, updateAvailable: updateAvailable)
        }
        let summary = quota.windows.map { "\($0.name)剩余 \($0.value)" }.joined(separator: "，")
        let current = selected.map { "环内显示\($0.name)" } ?? "暂无额度"
        let description = (preview ? "预览 · " : "") + current + " · " + summary + " · " + message + (updateAvailable ? " · 有新版本可更新" : "")
        button.toolTip = description
        button.setAccessibilityLabel("Codex HUD，" + description)
    }

    private func gauge(remaining: Double?, digits: String, loading: Bool, warning: Bool, updateAvailable: Bool) -> NSImage {
        let image = NSImage(size: NSSize(width: 22, height: 22), flipped: false) { _ in
            let center = NSPoint(x: 11, y: 11)
            let track = NSBezierPath(ovalIn: NSRect(x: 1, y: 1, width: 20, height: 20))
            track.lineWidth = 1.2
            if loading { track.setLineDash([1.5, 1.5], count: 2, phase: 0) }
            NSColor.black.withAlphaComponent(0.22).setStroke(); track.stroke()
            if let remaining, remaining > 0 {
                let progress = NSBezierPath()
                progress.appendArc(withCenter: center, radius: 10, startAngle: 90, endAngle: 90 - CGFloat(min(100, max(0, remaining))) * 3.6, clockwise: true)
                progress.lineWidth = 1.4; progress.lineCapStyle = .round
                NSColor.black.setStroke(); progress.stroke()
            }
            let font = NSFont.monospacedDigitSystemFont(ofSize: digits.count >= 3 ? 8 : 10, weight: .semibold)
            let attributes: [NSAttributedString.Key: Any] = [.font: font, .foregroundColor: NSColor.black]
            let text = digits as NSString
            let size = text.size(withAttributes: attributes)
            text.draw(at: NSPoint(x: (22 - size.width) / 2, y: (22 - size.height) / 2), withAttributes: attributes)
            // 更新或异常用环右上方的小标记提示，始终保留中央读数。
            if updateAvailable || warning {
                let marker = NSRect(x: 17.5, y: 17.5, width: 4, height: 4)
                NSGraphicsContext.saveGraphicsState()
                NSGraphicsContext.current?.compositingOperation = .copy
                NSColor.clear.setFill()
                NSBezierPath(ovalIn: marker.insetBy(dx: -0.8, dy: -0.8)).fill()
                NSGraphicsContext.restoreGraphicsState()
                NSColor.black.setFill(); NSBezierPath(ovalIn: marker).fill()
            }
            return true
        }
        // 模板图交由系统适配浅色、深色及菜单选中状态。
        image.isTemplate = true
        return image
    }
}
