import Foundation

// 所有状态更新都在主线程执行；刷新失败保留上次成功数据和时间。
final class QuotaStore {
    typealias Read = (@escaping (Result<[QuotaWindow], Error>) -> Void) -> Void
    private let read: Read
    private let now: () -> Double
    private(set) var snapshot = QuotaSnapshot(windows: [], loading: false)
    var changed: ((QuotaSnapshot) -> Void)?

    init(now: @escaping () -> Double = { Date().timeIntervalSince1970 * 1000 }, read: @escaping Read) {
        self.now = now; self.read = read
    }

    func refresh() {
        precondition(Thread.isMainThread)
        guard !snapshot.loading else { return }
        snapshot.loading = true; changed?(snapshot)
        read { [weak self] result in
            let finish = {
                guard let self else { return }
                switch result {
                case .success(let windows):
                    self.snapshot.windows = windows
                    self.snapshot.updatedAt = self.now()
                    self.snapshot.error = nil
                case .failure(let error):
                    self.snapshot.error = error.localizedDescription
                }
                self.snapshot.loading = false
                self.changed?(self.snapshot)
            }
            if Thread.isMainThread { finish() } else { DispatchQueue.main.async(execute: finish) }
        }
    }
}
