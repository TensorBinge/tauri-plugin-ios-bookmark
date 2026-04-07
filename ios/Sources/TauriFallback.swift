#if !os(iOS)
import Foundation

public class Logger {
  public static func debug(_ items: Any..., category: String = "app") {}
  public static func info(_ items: Any..., category: String = "app") {}
  public static func error(_ items: Any..., category: String = "app") {}
}

public class PluginManager {
  public static let shared = PluginManager()
  public var viewController: AnyObject?
}

open class Plugin: NSObject {
  public let manager: PluginManager = .shared
  @objc open func load(webview: AnyObject) {}
}

public class Invoke: NSObject {
  private let rawArgs: String

  public override init() {
    self.rawArgs = "{}"
    super.init()
  }

  public init(rawArgs: String) {
    self.rawArgs = rawArgs
    super.init()
  }

  public func parseArgs<T: Decodable>(_ type: T.Type) throws -> T {
    let jsonData = rawArgs.data(using: .utf8) ?? Data("{}".utf8)
    return try JSONDecoder().decode(type, from: jsonData)
  }

  public func resolve() {}

  public func resolve<T: Encodable>(_ data: T) {}

  public func reject(_ message: String, code: String? = nil, error: Error? = nil, data: Any? = nil) {}
}
#endif