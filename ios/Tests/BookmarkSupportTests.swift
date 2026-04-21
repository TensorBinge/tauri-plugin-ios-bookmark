import XCTest
@testable import tauri_plugin_ios_bookmark

final class BookmarkSupportTests: XCTestCase {
  func testDetailedErrorDescriptionIncludesDecodingContext() {
    let error = DecodingError.keyNotFound(
      AnyCodingKey("targetPath"),
      DecodingError.Context(
        codingPath: [AnyCodingKey("args")],
        debugDescription: "No value associated with key AnyCodingKey(stringValue: \"targetPath\", intValue: nil) (\"targetPath\").",
        underlyingError: nil
      )
    )

    let description = detailedErrorDescription(error)

    XCTAssertTrue(description.contains("DecodingError.keyNotFound(targetPath)"))
    XCTAssertTrue(description.contains("codingPath=args"))
    XCTAssertTrue(description.contains("No value associated with key"))
  }

  func testBookmarkRejectMessageUsesDetailedIoDescription() {
    let error = NSError(domain: NSCocoaErrorDomain, code: CocoaError.fileReadNoSuchFile.rawValue, userInfo: [
      NSDebugDescriptionErrorKey: "Missing file during coordinated list",
      NSFilePathErrorKey: "/private/var/mobile/Library/Mobile Documents/com~apple~CloudDocs/MarkScope",
    ])

    let message = bookmarkRejectMessage(for: error)

    XCTAssertTrue(message.hasPrefix("BOOKMARK_ERROR:NOT_FOUND:"))
    XCTAssertTrue(message.contains("Missing file during coordinated list"))
    XCTAssertTrue(message.contains("domain=NSCocoaErrorDomain"))
  }
}

private struct AnyCodingKey: CodingKey {
  let stringValue: String
  let intValue: Int?

  init(_ stringValue: String) {
    self.stringValue = stringValue
    self.intValue = nil
  }

  init?(stringValue: String) {
    self.init(stringValue)
  }

  init?(intValue: Int) {
    self.stringValue = String(intValue)
    self.intValue = intValue
  }
}