import Foundation
#if os(iOS)
import Tauri
#endif

let bookmarkNativeErrorPrefix = "BOOKMARK_ERROR"

enum BookmarkErrorCode: String {
  case unsupported = "UNSUPPORTED"
  case notFound = "NOT_FOUND"
  case stale = "STALE"
  case permissionDenied = "PERMISSION_DENIED"
  case ioError = "IO_ERROR"
  case cancelled = "CANCELLED"
  case targetMismatch = "TARGET_MISMATCH"
  case folderNotEmpty = "FOLDER_NOT_EMPTY"
  case nativeError = "NATIVE_ERROR"
}

struct BookmarkPluginError: LocalizedError {
  let code: BookmarkErrorCode
  let message: String

  var errorDescription: String? {
    message
  }
}

func bookmarkError(_ code: BookmarkErrorCode, _ message: String) -> BookmarkPluginError {
  BookmarkPluginError(code: code, message: message)
}

func detailedErrorDescription(_ error: Error) -> String {
  if let bookmarkError = error as? BookmarkPluginError {
    return "BookmarkPluginError(code=\(bookmarkError.code.rawValue), message=\(bookmarkError.message))"
  }

  if let decodingError = error as? DecodingError {
    switch decodingError {
    case let .dataCorrupted(context):
      return "DecodingError.dataCorrupted(debug=\(context.debugDescription), codingPath=\(formatCodingPath(context.codingPath))\(formatUnderlyingError(context.underlyingError)))"
    case let .keyNotFound(key, context):
      return "DecodingError.keyNotFound(\(key.stringValue))(debug=\(context.debugDescription), codingPath=\(formatCodingPath(context.codingPath))\(formatUnderlyingError(context.underlyingError)))"
    case let .valueNotFound(type, context):
      return "DecodingError.valueNotFound(\(String(describing: type)))(debug=\(context.debugDescription), codingPath=\(formatCodingPath(context.codingPath))\(formatUnderlyingError(context.underlyingError)))"
    case let .typeMismatch(type, context):
      return "DecodingError.typeMismatch(\(String(describing: type)))(debug=\(context.debugDescription), codingPath=\(formatCodingPath(context.codingPath))\(formatUnderlyingError(context.underlyingError)))"
    @unknown default:
      return "DecodingError.unknown(\(String(reflecting: decodingError)))"
    }
  }

  let nsError = error as NSError
  var components = [
    "domain=\(nsError.domain)",
    "code=\(nsError.code)",
    "description=\(nsError.localizedDescription)",
  ]

  if let debugDescription = nsError.userInfo[NSDebugDescriptionErrorKey] as? String, !debugDescription.isEmpty {
    components.append("debug=\(debugDescription)")
  }

  if let filePath = nsError.userInfo[NSFilePathErrorKey] as? String, !filePath.isEmpty {
    components.append("filePath=\(filePath)")
  }

  if let underlyingError = nsError.userInfo[NSUnderlyingErrorKey] as? NSError {
    components.append("underlying={\(detailedErrorDescription(underlyingError))}")
  }

  return components.joined(separator: ", ")
}

private func formatCodingPath(_ codingPath: [CodingKey]) -> String {
  if codingPath.isEmpty {
    return "<root>"
  }

  return codingPath.map { key in
    if let intValue = key.intValue {
      return "[\(intValue)]"
    }

    return key.stringValue
  }.joined(separator: ".")
}

private func formatUnderlyingError(_ error: Error?) -> String {
  guard let error else {
    return ""
  }

  return ", underlying={\(detailedErrorDescription(error))}"
}

func bookmarkRejectMessage(for error: Error) -> String {
  if let bookmarkError = error as? BookmarkPluginError {
    return "\(bookmarkNativeErrorPrefix):\(bookmarkError.code.rawValue):\(bookmarkError.message)"
  }

  let nsError = error as NSError
  let detail = detailedErrorDescription(error)
  if nsError.domain == NSCocoaErrorDomain {
    switch CocoaError.Code(rawValue: nsError.code) {
    case .fileNoSuchFile:
      return "\(bookmarkNativeErrorPrefix):\(BookmarkErrorCode.notFound.rawValue):\(detail)"
    case .fileReadNoPermission, .fileWriteNoPermission:
      return "\(bookmarkNativeErrorPrefix):\(BookmarkErrorCode.permissionDenied.rawValue):\(detail)"
    default:
      return "\(bookmarkNativeErrorPrefix):\(BookmarkErrorCode.ioError.rawValue):\(detail)"
    }
  }

  return "\(bookmarkNativeErrorPrefix):\(BookmarkErrorCode.nativeError.rawValue):\(detail)"
}

struct PickResultDTO: Encodable {
  let bookmarkId: String
  let fileName: String
  let filePath: String
  let content: String
}

struct PickFolderResultDTO: Encodable {
  let bookmarkId: String
  let folderName: String
  let folderPath: String
}

struct ReadResultDTO: Encodable {
  let fileName: String
  let filePath: String
  let content: String
}

struct FolderBookmarkEntryDTO: Encodable {
  let name: String
  let path: String
  let isDir: Bool
  let size: UInt64?
  let mtime: UInt64?
}

@_cdecl("init_plugin_ios_bookmark")
func initPluginIosBookmark() -> Plugin {
  BookmarkPlugin()
}
