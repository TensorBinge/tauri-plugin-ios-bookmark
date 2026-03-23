import Foundation
import Tauri
import UIKit
import UniformTypeIdentifiers

final class BookmarkPlugin: Plugin, UIDocumentPickerDelegate, UIAdaptivePresentationControllerDelegate {
  private var pendingInvoke: Invoke?
  private let store = BookmarkStore()

  @objc public func pickAndBookmark(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: pickAndBookmark entered mainThread=\(Thread.isMainThread)", category: "ios-bookmark")

    let presentPicker = {
      Logger.info("[ios-bookmark] swift plugin: pickAndBookmark on main thread", category: "ios-bookmark")
      guard let presenter = self.manager.viewController else {
        Logger.error("[ios-bookmark] swift plugin: no active view controller", category: "ios-bookmark")
        invoke.reject("\(bookmarkNativeErrorPrefix):\(BookmarkErrorCode.nativeError.rawValue):No active view controller")
        return
      }

      guard self.pendingInvoke == nil else {
        Logger.error("[ios-bookmark] swift plugin: picker already in progress", category: "ios-bookmark")
        invoke.reject("\(bookmarkNativeErrorPrefix):\(BookmarkErrorCode.nativeError.rawValue):Picker already in progress")
        return
      }

      let picker = UIDocumentPickerViewController(
        forOpeningContentTypes: self.markdownTypes(),
        asCopy: false
      )
      picker.delegate = self
      picker.allowsMultipleSelection = false
      picker.modalPresentationStyle = .fullScreen
      picker.presentationController?.delegate = self
      self.pendingInvoke = invoke
      Logger.info("[ios-bookmark] swift plugin: presenting UIDocumentPickerViewController", category: "ios-bookmark")
      presenter.present(picker, animated: true, completion: nil)
    }

    if Thread.isMainThread {
      presentPicker()
    } else {
      DispatchQueue.main.async(execute: presentPicker)
    }
  }

  public func documentPicker(
    _ controller: UIDocumentPickerViewController,
    didPickDocumentsAt urls: [URL]
  ) {
    Logger.info("[ios-bookmark] swift plugin: didPickDocumentsAt count=\(urls.count)", category: "ios-bookmark")
    guard let url = urls.first else {
      Logger.error("[ios-bookmark] swift plugin: didPickDocumentsAt without url", category: "ios-bookmark")
      pendingInvoke?.reject("\(bookmarkNativeErrorPrefix):\(BookmarkErrorCode.cancelled.rawValue):No file selected")
      pendingInvoke = nil
      return
    }

    guard url.startAccessingSecurityScopedResource() else {
      Logger.error("[ios-bookmark] swift plugin: startAccessingSecurityScopedResource failed for \(url)", category: "ios-bookmark")
      pendingInvoke?.reject("\(bookmarkNativeErrorPrefix):\(BookmarkErrorCode.permissionDenied.rawValue):Failed to access security-scoped resource")
      pendingInvoke = nil
      return
    }
    defer { url.stopAccessingSecurityScopedResource() }

    do {
      let bookmarkData = try url.bookmarkData(
        options: [],
        includingResourceValuesForKeys: nil,
        relativeTo: nil
      )
      let fileName = url.lastPathComponent
      let content = try coordinatedRead(url: url)
      let id = store.save(bookmarkData: bookmarkData, fileName: fileName)
      Logger.info("[ios-bookmark] swift plugin: resolving pickAndBookmark for \(fileName)", category: "ios-bookmark")

      pendingInvoke?.resolve(
        PickResultDTO(bookmarkId: id, fileName: fileName, filePath: url.path, content: content)
      )
    } catch {
      Logger.error("[ios-bookmark] swift plugin: pickAndBookmark error \(error.localizedDescription)", category: "ios-bookmark")
      pendingInvoke?.reject(bookmarkRejectMessage(for: error))
    }

    pendingInvoke = nil
  }

  public func documentPickerWasCancelled(_ controller: UIDocumentPickerViewController) {
    Logger.info("[ios-bookmark] swift plugin: documentPickerWasCancelled", category: "ios-bookmark")
    pendingInvoke?.reject("\(bookmarkNativeErrorPrefix):\(BookmarkErrorCode.cancelled.rawValue):User cancelled")
    pendingInvoke = nil
  }

  public func presentationControllerDidDismiss(_ presentationController: UIPresentationController) {
    Logger.info("[ios-bookmark] swift plugin: presentationControllerDidDismiss", category: "ios-bookmark")
    pendingInvoke?.reject("\(bookmarkNativeErrorPrefix):\(BookmarkErrorCode.cancelled.rawValue):Picker dismissed")
    pendingInvoke = nil
  }

  @objc public func readByBookmark(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: readByBookmark entered", category: "ios-bookmark")
    struct Args: Decodable {
      let id: String
    }

    do {
      let args = try invoke.parseArgs(Args.self)
      guard let bookmarkData = store.getBookmarkData(id: args.id) else {
        throw bookmarkError(.notFound, "No bookmark found for id: \(args.id)")
      }

      var stale = false
      let url = try URL(
        resolvingBookmarkData: bookmarkData,
        options: [],
        relativeTo: nil,
        bookmarkDataIsStale: &stale
      )

      guard url.startAccessingSecurityScopedResource() else {
        throw bookmarkError(.permissionDenied, "Failed to access security-scoped resource for id: \(args.id)")
      }
      defer { url.stopAccessingSecurityScopedResource() }

      if stale {
        let refreshedBookmark = try url.bookmarkData(
          options: [],
          includingResourceValuesForKeys: nil,
          relativeTo: nil
        )
        let refreshedName = try? url.resourceValues(forKeys: [.nameKey]).name
        store.update(id: args.id, bookmarkData: refreshedBookmark, fileName: refreshedName)
      }

      let content = try coordinatedRead(url: url)
      let fileName = store.getFileName(id: args.id) ?? url.lastPathComponent
      Logger.info("[ios-bookmark] swift plugin: readByBookmark resolved for \(fileName)", category: "ios-bookmark")
      invoke.resolve(ReadResultDTO(fileName: fileName, content: content))
    } catch {
      Logger.error("[ios-bookmark] swift plugin: readByBookmark error \(error.localizedDescription)", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

  @objc public func forgetBookmark(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: forgetBookmark entered", category: "ios-bookmark")
    struct Args: Decodable {
      let id: String
    }

    do {
      let args = try invoke.parseArgs(Args.self)
      store.remove(id: args.id)
      Logger.info("[ios-bookmark] swift plugin: forgetBookmark resolved", category: "ios-bookmark")
      invoke.resolve()
    } catch {
      Logger.error("[ios-bookmark] swift plugin: forgetBookmark error \(error.localizedDescription)", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

  private func coordinatedRead(url: URL) throws -> String {
    Logger.info("[ios-bookmark] swift plugin: coordinatedRead start for \(url.lastPathComponent)", category: "ios-bookmark")
    var content: String?
    var coordinatorError: NSError?
    var readError: Error?
    let coordinator = NSFileCoordinator(filePresenter: nil)

    coordinator.coordinate(readingItemAt: url, options: [], error: &coordinatorError) { readUrl in
      do {
        content = try String(contentsOf: readUrl, encoding: .utf8)
      } catch {
        readError = error
      }
    }

    if let coordinatorError {
      throw coordinatorError
    }
    if let readError {
      throw readError
    }
    guard let content else {
      throw bookmarkError(.ioError, "Failed to read file content")
    }

    Logger.info("[ios-bookmark] swift plugin: coordinatedRead success for \(url.lastPathComponent)", category: "ios-bookmark")

    return content
  }

  private func markdownTypes() -> [UTType] {
    var types: [UTType] = [.plainText, .text]
    if let markdown = UTType(filenameExtension: "md") {
      types.append(markdown)
    }
    if let markdownAlt = UTType(filenameExtension: "markdown") {
      types.append(markdownAlt)
    }
    return Array(Set(types))
  }
}
