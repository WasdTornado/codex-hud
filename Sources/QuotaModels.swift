import Foundation

struct QuotaWindow: Codable, Equatable {
    var minutes: Int?
    var remaining: Double?
    var resetsAt: Double?
    var shortName: String { minutes == 300 ? "5h" : minutes == 10080 ? "周" : "额度" }
    var name: String { minutes == 300 ? "每 5 小时" : minutes == 10080 ? "每周" : minutes.map { "\($0) 分钟" } ?? "当前窗口" }
    var value: String { remaining.map { "\(Int(max(0, min(100, $0)).rounded()))%" } ?? "—" }
}

struct QuotaSnapshot: Codable, Equatable {
    var windows: [QuotaWindow]
    var updatedAt: Double?
    var loading: Bool
    var error: String?
    static let empty = QuotaSnapshot(windows: [], loading: true)
}


enum QuotaRefreshPolicy {
    static let interval: TimeInterval = 300
    static func isStale(updatedAt: Double?, now: Double, hasError: Bool) -> Bool {
        hasError || updatedAt.map { now - $0 / 1000 > interval + 30 } == true
    }
}
