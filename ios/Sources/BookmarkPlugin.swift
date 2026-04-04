import Foundation
import Tauri
import UIKit
import UniformTypeIdentifiers

private struct PickBookmarkArgs: Decodable {
  let request: PickBookmarkRequestDTO?
}

private struct PickFolderBookmarkArgs: Decodable {
  let request: PickFolderBookmarkRequestDTO?
}

private struct PickBookmarkRequestDTO: Decodable {
  let targetPath: String?
  let suggestedFileName: String?
}

private struct PickFolderBookmarkRequestDTO: Decodable {
  let targetPath: String?
}

private enum PendingPickerKind {
  case file
  case folder
}

final class BookmarkPlugin: Plugin, UIDocumentPickerDelegate, UIAdaptivePresentationControllerDelegate {
  private var pendingInvoke: Invoke?
  private var pendingPickRequest: PickBookmarkRequestDTO?
  private var pendingFolderPickRequest: PickFolderBookmarkRequestDTO?
  private var pendingPickerKind: PendingPickerKind?
  private let store = BookmarkStore()

  @objc public func pickAndBookmark(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: pickAndBookmark entered mainThread=\(Thread.isMainThread)", category: "ios-bookmark")

    let args = (try? invoke.parseArgs(PickBookmarkArgs.self)) ?? PickBookmarkArgs(request: nil)

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
      self.pendingPickRequest = args.request
      self.pendingPickerKind = .file

      if let directoryUrl = self.initialDirectoryUrl(for: args.request) {
        picker.directoryURL = directoryUrl
      }

      Logger.info("[ios-bookmark] swift plugin: presenting UIDocumentPickerViewController", category: "ios-bookmark")
      presenter.present(picker, animated: true, completion: nil)
    }

    if Thread.isMainThread {
      presentPicker()
    } else {
      DispatchQueue.main.async(execute: presentPicker)
    }
  }

  @objc public func pickFolderAndBookmark(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: pickFolderAndBookmark entered mainThread=\(Thread.isMainThread)", category: "ios-bookmark")

    let args = (try? invoke.parseArgs(PickFolderBookmarkArgs.self)) ?? PickFolderBookmarkArgs(request: nil)

    let presentPicker = {
      Logger.info("[ios-bookmark] swift plugin: pickFolderAndBookmark on main thread", category: "ios-bookmark")
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
        forOpeningContentTypes: [.folder],
        asCopy: false
      )
      picker.delegate = self
      picker.allowsMultipleSelection = false
      picker.modalPresentationStyle = .fullScreen
      picker.presentationController?.delegate = self
      self.pendingInvoke = invoke
      self.pendingFolderPickRequest = args.request
      self.pendingPickerKind = .folder

      if let directoryUrl = self.initialDirectoryUrl(for: args.request) {
        picker.directoryURL = directoryUrl
      }

      Logger.info("[ios-bookmark] swift plugin: presenting folder UIDocumentPickerViewController", category: "ios-bookmark")
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
      clearPendingPickState()
      return
    }

    switch pendingPickerKind {
    case .folder:
      resolvePickedFolder(url)
    case .file, .none:
      resolvePickedFile(url)
    }
  }

  @objc public func readByFolderBookmark(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: readByFolderBookmark entered", category: "ios-bookmark")
    struct Args: Decodable {
      let id: String
      let targetPath: String
    }

    do {
      let args = try invoke.parseArgs(Args.self)
      guard let bookmarkData = store.getBookmarkData(id: args.id) else {
        throw bookmarkError(.notFound, "No folder bookmark found for id: \(args.id)")
      }

      var stale = false
      let folderUrl = try URL(
        resolvingBookmarkData: bookmarkData,
        options: [],
        relativeTo: nil,
        bookmarkDataIsStale: &stale
      )

      guard urlIsWithinFolder(targetPath: args.targetPath, folderPath: folderUrl.path) else {
        throw bookmarkError(.permissionDenied, "Requested path is outside the bookmarked folder")
      }

      guard folderUrl.startAccessingSecurityScopedResource() else {
        throw bookmarkError(.permissionDenied, "Failed to access security-scoped folder for id: \(args.id)")
      }
      defer { folderUrl.stopAccessingSecurityScopedResource() }

      if stale {
        let refreshedBookmark = try folderUrl.bookmarkData(
          options: [],
          includingResourceValuesForKeys: nil,
          relativeTo: nil
        )
        let refreshedName = try? folderUrl.resourceValues(forKeys: [.nameKey]).name
        store.updateFolder(id: args.id, bookmarkData: refreshedBookmark, folderName: refreshedName)
      }

      let targetUrl = URL(fileURLWithPath: args.targetPath).resolvingSymlinksInPath()
      let resolvedFolderPath = folderUrl.resolvingSymlinksInPath().path
      let normalizedTarget = normalizeIosScopedPath(targetUrl.path)
      let normalizedFolder = normalizeIosScopedPath(resolvedFolderPath)
      guard resolvedPathIsWithinFolder(normalizedTarget, folder: normalizedFolder) else {
        throw bookmarkError(
          .permissionDenied,
          "Resolved path '\(normalizedTarget)' is outside the bookmarked folder '\(normalizedFolder)'"
        )
      }
      let content = try coordinatedRead(url: targetUrl)
      Logger.info("[ios-bookmark] swift plugin: readByFolderBookmark resolved for \(targetUrl.lastPathComponent)", category: "ios-bookmark")
      invoke.resolve(ReadResultDTO(fileName: targetUrl.lastPathComponent, filePath: targetUrl.path, content: content))
    } catch {
      Logger.error("[ios-bookmark] swift plugin: readByFolderBookmark error \(error.localizedDescription)", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

  private func resolvePickedFile(_ url: URL) {
    if let targetPath = pendingPickRequest?.targetPath,
       !pathsMatch(selectedUrl: url, targetPath: targetPath) {
      Logger.error("[ios-bookmark] swift plugin: selected file does not match requested target", category: "ios-bookmark")
      rejectPendingInvoke(bookmarkError(.targetMismatch, "Selected file does not match requested target"))
      return
    }

    guard url.startAccessingSecurityScopedResource() else {
      Logger.error("[ios-bookmark] swift plugin: startAccessingSecurityScopedResource failed for \(url)", category: "ios-bookmark")
      rejectPendingInvoke(bookmarkError(.permissionDenied, "Failed to access security-scoped resource"))
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
      rejectPendingInvoke(error)
      return
    }

    clearPendingPickState()
  }

  private func resolvePickedFolder(_ url: URL) {
    if let targetPath = pendingFolderPickRequest?.targetPath,
       !pathsMatch(selectedUrl: url, targetPath: targetPath) {
      Logger.error("[ios-bookmark] swift plugin: selected folder does not match requested target", category: "ios-bookmark")
      rejectPendingInvoke(bookmarkError(.targetMismatch, "Selected folder does not match requested target"))
      return
    }

    guard url.startAccessingSecurityScopedResource() else {
      Logger.error("[ios-bookmark] swift plugin: startAccessingSecurityScopedResource failed for folder \(url)", category: "ios-bookmark")
      rejectPendingInvoke(bookmarkError(.permissionDenied, "Failed to access security-scoped folder"))
      return
    }
    defer { url.stopAccessingSecurityScopedResource() }

    do {
      let bookmarkData = try url.bookmarkData(
        options: [],
        includingResourceValuesForKeys: nil,
        relativeTo: nil
      )
      let folderName = url.lastPathComponent.isEmpty ? url.path : url.lastPathComponent
      let id = store.saveFolder(bookmarkData: bookmarkData, folderName: folderName)
      Logger.info("[ios-bookmark] swift plugin: resolving pickFolderAndBookmark for \(folderName)", category: "ios-bookmark")

      pendingInvoke?.resolve(
        PickFolderResultDTO(bookmarkId: id, folderName: folderName, folderPath: url.path)
      )
    } catch {
      Logger.error("[ios-bookmark] swift plugin: pickFolderAndBookmark error \(error.localizedDescription)", category: "ios-bookmark")
      rejectPendingInvoke(error)
      return
    }

    clearPendingPickState()
  }

  public func documentPickerWasCancelled(_ controller: UIDocumentPickerViewController) {
    Logger.info("[ios-bookmark] swift plugin: documentPickerWasCancelled", category: "ios-bookmark")
    rejectPendingInvoke(bookmarkError(.cancelled, "User cancelled"))
  }

  public func presentationControllerDidDismiss(_ presentationController: UIPresentationController) {
    Logger.info("[ios-bookmark] swift plugin: presentationControllerDidDismiss", category: "ios-bookmark")
    rejectPendingInvoke(bookmarkError(.cancelled, "Picker dismissed"))
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
      invoke.resolve(ReadResultDTO(fileName: fileName, filePath: url.path, content: content))
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

  private func initialDirectoryUrl(for request: PickBookmarkRequestDTO?) -> URL? {
    guard let targetPath = request?.targetPath else {
      return nil
    }

    let directoryPath = (targetPath as NSString).deletingLastPathComponent
    guard !directoryPath.isEmpty else {
      return nil
    }

    return URL(fileURLWithPath: directoryPath, isDirectory: true)
  }

  private func initialDirectoryUrl(for request: PickFolderBookmarkRequestDTO?) -> URL? {
    guard let targetPath = request?.targetPath, !targetPath.isEmpty else {
      return nil
    }

    return URL(fileURLWithPath: targetPath, isDirectory: true)
  }

  private func pathsMatch(selectedUrl: URL, targetPath: String) -> Bool {
    standardizedPath(for: selectedUrl.path) == standardizedPath(for: targetPath)
  }

  private func standardizedPath(for path: String) -> String {
    normalizeIosScopedPath(URL(fileURLWithPath: path).standardizedFileURL.path)
  }

  private func normalizeIosScopedPath(_ path: String) -> String {
    let normalizedPath = path == "/" ? path : path.replacingOccurrences(of: #"/+$"#, with: "", options: .regularExpression)

    guard normalizedPath.hasPrefix("/private/var/") else {
      return normalizedPath
    }

    return String(normalizedPath.dropFirst("/private".count))
  }

  private func urlIsWithinFolder(targetPath: String, folderPath: String) -> Bool {
    let normalizedTarget = normalizeIosScopedPath(
      URL(fileURLWithPath: targetPath).resolvingSymlinksInPath().path
    )
    let normalizedFolder = normalizeIosScopedPath(
      URL(fileURLWithPath: folderPath).resolvingSymlinksInPath().path
    )
    return resolvedPathIsWithinFolder(normalizedTarget, folder: normalizedFolder)
  }

  private func resolvedPathIsWithinFolder(_ normalizedTarget: String, folder normalizedFolder: String) -> Bool {
    if normalizedFolder == "/" {
      return normalizedTarget.hasPrefix("/")
    }
    return normalizedTarget == normalizedFolder || normalizedTarget.hasPrefix(normalizedFolder + "/")
  }

  private func rejectPendingInvoke(_ error: Error) {
    pendingInvoke?.reject(bookmarkRejectMessage(for: error))
    clearPendingPickState()
  }

  private func clearPendingPickState() {
    pendingInvoke = nil
    pendingPickRequest = nil
    pendingFolderPickRequest = nil
    pendingPickerKind = nil
  }
}
