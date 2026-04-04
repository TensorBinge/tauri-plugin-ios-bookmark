import Foundation
import Tauri

let bookmarkNativeErrorPrefix = "BOOKMARK_ERROR"

enum BookmarkErrorCode: String {
  case unsupported = "UNSUPPORTED"
  case notFound = "NOT_FOUND"
  case stale = "STALE"
  case permissionDenied = "PERMISSION_DENIED"
  case ioError = "IO_ERROR"
  case cancelled = "CANCELLED"
  case targetMismatch = "TARGET_MISMATCH"
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

func bookmarkRejectMessage(for error: Error) -> String {
  if let bookmarkError = error as? BookmarkPluginError {
    return "\(bookmarkNativeErrorPrefix):\(bookmarkError.code.rawValue):\(bookmarkError.message)"
  }

  let nsError = error as NSError
  if nsError.domain == NSCocoaErrorDomain {
    switch CocoaError.Code(rawValue: nsError.code) {
    case .fileNoSuchFile:
      return "\(bookmarkNativeErrorPrefix):\(BookmarkErrorCode.notFound.rawValue):\(nsError.localizedDescription)"
    case .fileReadNoPermission, .fileWriteNoPermission:
      return "\(bookmarkNativeErrorPrefix):\(BookmarkErrorCode.permissionDenied.rawValue):\(nsError.localizedDescription)"
    default:
      return "\(bookmarkNativeErrorPrefix):\(BookmarkErrorCode.ioError.rawValue):\(nsError.localizedDescription)"
    }
  }

  return "\(bookmarkNativeErrorPrefix):\(BookmarkErrorCode.nativeError.rawValue):\(error.localizedDescription)"
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

@_cdecl("init_plugin_ios_bookmark")
func initPluginIosBookmark() -> Plugin {
  BookmarkPlugin()
}
