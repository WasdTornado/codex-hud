import Foundation
import CoreFoundation
import Darwin

enum QuotaError: LocalizedError {
    case missingCLI, launch, connection, timeout, cancelled
    var errorDescription: String? {
        switch self {
        case .missingCLI: return "未找到 Codex，请先安装并登录 Codex 应用或 CLI"
        case .launch: return "无法启动额度读取程序"
        case .connection: return "同步失败，请检查网络或 Codex 登录"
        case .timeout: return "额度读取超时，请稍后重试"
        case .cancelled: return "额度读取已取消"
        }
    }
}

final class QuotaClient {
    private let lock = NSLock()
    private var active: Process?
    private var cancelled = false

    private static func number(_ value: Any?) -> Double? {
        guard let n = value as? NSNumber, CFGetTypeID(n) != CFBooleanGetTypeID(), n.doubleValue.isFinite else { return nil }
        return n.doubleValue
    }

    static func normalize(_ response: [String: Any]) -> [QuotaWindow] {
        let bucket: [String: Any]?
        if let map = response["rateLimitsByLimitId"] as? [String: Any], map.values.contains(where: { $0 is [String: Any] }) {
            bucket = map["codex"] as? [String: Any]
        } else { bucket = response["rateLimits"] as? [String: Any] }
        guard let bucket else { return [] }
        let windows = ["primary", "secondary"].compactMap { key -> QuotaWindow? in
            guard let window = bucket[key] as? [String: Any] else { return nil }
            let duration = number(window["windowDurationMins"])
            let minutes = duration.flatMap { value in value > 0 && value < Double(Int.max) && value.rounded() == value ? Int(value) : nil }
            return QuotaWindow(minutes: minutes, remaining: number(window["usedPercent"]).map { max(0, min(100, 100 - $0)) }, resetsAt: number(window["resetsAt"]))
        }
        // 窗口时长决定顺序，不依据套餐名或接口中的主次位置推断。
        func priority(_ window: QuotaWindow) -> Int { window.minutes == 300 ? 0 : window.minutes == 10080 ? 1 : 2 }
        return windows.enumerated().sorted {
            priority($0.element) == priority($1.element) ? $0.offset < $1.offset : priority($0.element) < priority($1.element)
        }.map(\.element)
    }

    static func findExecutable() -> URL? {
        let home = FileManager.default.homeDirectoryForCurrentUser.path
        var paths = (ProcessInfo.processInfo.environment["PATH"] ?? "").split(separator: ":").map { "\($0)/codex" }
        for base in ["/Applications", home + "/Applications"] {
            for app in ["Codex", "ChatGPT"] {
                paths.append("\(base)/\(app).app/Contents/Resources/codex-cli/CodexCLI.app/Contents/MacOS/codex")
                paths.append("\(base)/\(app).app/Contents/Resources/codex")
            }
        }
        paths += ["/opt/homebrew/bin/codex", "/usr/local/bin/codex", home + "/.volta/bin/codex"]
        return paths.first { path in
            var directory: ObjCBool = false
            return FileManager.default.fileExists(atPath: path, isDirectory: &directory) && !directory.boolValue && FileManager.default.isExecutableFile(atPath: path)
        }.map { URL(fileURLWithPath: $0) }
    }

    func cancel() {
        lock.lock(); cancelled = true; let task = active; lock.unlock()
        if let task, task.isRunning { task.terminate() }
    }

    func read() throws -> [QuotaWindow] {
        guard let executable = Self.findExecutable() else { throw QuotaError.missingCLI }
        return try read(executable: executable, arguments: ["app-server", "--listen", "stdio://"], timeout: 25)
    }

    func read(executable: URL, arguments: [String], timeout: TimeInterval) throws -> [QuotaWindow] {
        let process = Process(), input = Pipe(), output = Pipe()
        process.executableURL = executable; process.arguments = arguments
        // 子进程从应用支持目录运行，不依赖桌面源码，也不读取原版 Meter。
        let directory = FileManager.default.homeDirectoryForCurrentUser.appendingPathComponent("Library/Application Support/QuotaOrb")
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        process.currentDirectoryURL = directory
        var environment = ProcessInfo.processInfo.environment
        environment["PWD"] = directory.path; process.environment = environment
        process.standardInput = input; process.standardOutput = output; process.standardError = FileHandle.nullDevice
        lock.lock()
        if active != nil { lock.unlock(); throw QuotaError.connection }
        if cancelled { lock.unlock(); throw QuotaError.cancelled }
        do { try process.run(); active = process; lock.unlock() }
        catch { lock.unlock(); throw QuotaError.launch }
        try? input.fileHandleForReading.close(); try? output.fileHandleForWriting.close()
        let writeFD = input.fileHandleForWriting.fileDescriptor
        _ = fcntl(writeFD, F_SETNOSIGPIPE, 1)
        defer {
            try? input.fileHandleForWriting.close(); try? output.fileHandleForReading.close()
            if process.isRunning {
                process.terminate()
                // 退出也有上限，避免异常子进程长期驻留。
                for _ in 0..<20 { if !process.isRunning { break }; usleep(10_000) }
                if process.isRunning { kill(process.processIdentifier, SIGKILL) }
            }
            lock.lock(); active = nil; lock.unlock()
        }
        func send(_ message: [String: Any]) throws {
            var data = try JSONSerialization.data(withJSONObject: message); data.append(10)
            try data.withUnsafeBytes { buffer in
                var offset = 0
                while offset < buffer.count {
                    let written = Darwin.write(writeFD, buffer.baseAddress!.advanced(by: offset), buffer.count - offset)
                    if written < 0 && errno == EINTR { continue }
                    guard written > 0 else { throw QuotaError.connection }
                    offset += written
                }
            }
        }
        try send(["id": 1, "method": "initialize", "params": ["clientInfo": ["name": "codex_hud", "title": "Codex HUD", "version": Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? "1.0.0"], "capabilities": NSNull()]])
        let deadline = ProcessInfo.processInfo.systemUptime + timeout
        var pending = Data(), initialized = false
        var bytes = [UInt8](repeating: 0, count: 4096)
        while true {
            lock.lock(); let stopped = cancelled; lock.unlock()
            if stopped { throw QuotaError.cancelled }
            if ProcessInfo.processInfo.systemUptime >= deadline { throw QuotaError.timeout }
            var descriptor = pollfd(fd: output.fileHandleForReading.fileDescriptor, events: Int16(POLLIN), revents: 0)
            let ready = poll(&descriptor, 1, 50)
            if ready < 0 && errno == EINTR { continue }
            guard ready >= 0 else { throw QuotaError.connection }
            if ready == 0 { continue }
            let count = Darwin.read(descriptor.fd, &bytes, bytes.count)
            guard count > 0 else { throw QuotaError.connection }
            pending.append(contentsOf: bytes.prefix(count))
            guard pending.count < 1_048_576 else { throw QuotaError.connection }
            while let newline = pending.firstIndex(of: 10) {
                let line = pending.prefix(upTo: newline); pending.removeSubrange(...newline)
                guard let message = (try? JSONSerialization.jsonObject(with: line)) as? [String: Any] else { continue }
                if let method = message["method"] as? String {
                    if let id = message["id"] { try send(["id": id, "error": ["code": -32601, "message": "Unsupported method: \(method)"]]) }
                    continue
                }
                guard let id = Self.number(message["id"]), id == 1 || id == 2 else { continue }
                guard message["error"] == nil, let result = message["result"] as? [String: Any] else { throw QuotaError.connection }
                if id == 1 && !initialized {
                    initialized = true
                    try send(["method": "initialized"])
                    try send(["id": 2, "method": "account/rateLimits/read"])
                } else if id == 2 && initialized { return Self.normalize(result) }
            }
        }
    }
}
