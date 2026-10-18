import Foundation
@main struct QuotaCoreTests {
    static var failures = 0
    static var checks = 0
    static func check(_ condition: Bool, _ message: String) {
        checks += 1
        if !condition { failures += 1; print("失败：\(message)") }
    }
    static func main() {
        check(!QuotaRefreshPolicy.isStale(updatedAt: 1000, now: 300, hasError: false), "五分钟刷新期间不误报过期")
        check(!QuotaRefreshPolicy.isStale(updatedAt: 1000, now: 326, hasError: false), "下一次请求超时前保留正常等待状态")
        check(QuotaRefreshPolicy.isStale(updatedAt: 1000, now: 332, hasError: false), "超过刷新周期和请求宽限后提示过期")
        check(QuotaRefreshPolicy.isStale(updatedAt: 1000, now: 2, hasError: true), "刚更新后请求失败也立即提示")
        let fixture = CommandLine.arguments[1]
        let bucket: [String: Any] = ["primary": ["windowDurationMins": 10080, "usedPercent": 53, "resetsAt": 9000], "secondary": ["windowDurationMins": 300, "usedPercent": 18, "resetsAt": 2000]]
        let windows = QuotaClient.normalize(["rateLimitsByLimitId": ["codex": bucket, "code_review": ["primary": ["usedPercent": 99]]]])
        check(windows.map(\.minutes) == [300, 10080], "5h 在周额度前面")
        check(windows.map(\.remaining) == [82, 47], "使用量转换为剩余量")
        check(windows.map(\.resetsAt) == [2000, 9000], "保留对应重置时间")
        let values: [(Any, Double?)] = [(NSNull(), nil), (100, 0), (0, 100), (120, 0), (-10, 100), (true, nil)]
        for (raw, expected) in values {
            let result = QuotaClient.normalize(["rateLimits": ["primary": ["windowDurationMins": 300, "usedPercent": raw]]])
            check(result.count == 1 && result.first?.remaining == expected, "边界值 \(raw)")
        }
        check(QuotaClient.normalize(["rateLimitsByLimitId": ["code_review": bucket]]).isEmpty, "不冒用代码审查额度")
        check(QuotaClient.normalize([:]).isEmpty, "缺失额度保持为空")
        let client = QuotaClient()
        do {
            let result = try client.read(executable: URL(fileURLWithPath: "/usr/bin/python3"), arguments: [fixture], timeout: 2)
            check(result.map(\.remaining) == [82, 47], "真实子进程协议与噪音处理")
        } catch { check(false, "协议成功路径：\(error)") }
        do {
            _ = try client.read(executable: URL(fileURLWithPath: "/usr/bin/python3"), arguments: [fixture, "error"], timeout: 2)
            check(false, "服务错误必须失败")
        } catch { check(!error.localizedDescription.contains("secret-token") && error.localizedDescription.contains("登录"), "错误不泄露服务端内容") }
        let start = Date()
        do {
            _ = try client.read(executable: URL(fileURLWithPath: "/usr/bin/python3"), arguments: [fixture, "timeout"], timeout: 0.2)
            check(false, "请求必须超时")
        } catch { check(error.localizedDescription.contains("超时") && Date().timeIntervalSince(start) < 2, "超时退出") }
        do {
            _ = try client.read(executable: URL(fileURLWithPath: "/nonexistent/quota-test"), arguments: [], timeout: 1)
            check(false, "不存在的程序必须失败")
        } catch { check(error.localizedDescription.contains("启动"), "启动错误") }
        var reads = 0
        var completion: ((Result<[QuotaWindow], Error>) -> Void)?
        var time = 1000.0
        let store = QuotaStore(now: { time }) { callback in reads += 1; completion = callback }
        store.refresh(); store.refresh()
        check(reads == 1 && store.snapshot.loading, "连续刷新只发起一次读取")
        completion?(.success(windows))
        check(store.snapshot.windows == windows && store.snapshot.updatedAt == 1000 && !store.snapshot.loading, "成功保存额度和时间")
        time = 2000; store.refresh(); completion?(.failure(QuotaError.timeout))
        check(store.snapshot.windows == windows && store.snapshot.updatedAt == 1000 && store.snapshot.error != nil, "失败保留真实额度和成功时间")
        store.refresh(); completion?(.success([QuotaWindow(minutes: 300, remaining: 0)]))
        check(store.snapshot.windows.count == 1 && store.snapshot.windows[0].remaining == 0 && store.snapshot.updatedAt == 2000 && store.snapshot.error == nil, "双转单并从失败恢复，零额度不丢失")
        print("\(checks - failures)/\(checks) 项检查通过")
        exit(failures == 0 ? 0 : 1)
    }
}
