import Foundation

func normalizeIosScopedPath(_ path: String) -> String {
  let normalizedPath = path == "/" ? path : path.replacingOccurrences(of: #"/+$"#, with: "", options: .regularExpression)

  guard normalizedPath.hasPrefix("/private/var/") else {
    return normalizedPath
  }

  return String(normalizedPath.dropFirst("/private".count))
}

func resolvedPathIsWithinFolder(_ normalizedTarget: String, folder normalizedFolder: String) -> Bool {
  if normalizedFolder == "/" {
    return normalizedTarget.hasPrefix("/")
  }

  return normalizedTarget == normalizedFolder || normalizedTarget.hasPrefix(normalizedFolder + "/")
}

func urlIsWithinFolder(targetPath: String, folderPath: String) -> Bool {
  let normalizedTarget = normalizeIosScopedPath(
    URL(fileURLWithPath: targetPath).resolvingSymlinksInPath().path
  )
  let normalizedFolder = normalizeIosScopedPath(
    URL(fileURLWithPath: folderPath).resolvingSymlinksInPath().path
  )
  return resolvedPathIsWithinFolder(normalizedTarget, folder: normalizedFolder)
}

func resolveScopedChildUrlInBookmarkScope(targetPath: String, folderUrl: URL) throws -> URL {
  guard urlIsWithinFolder(targetPath: targetPath, folderPath: folderUrl.path) else {
    throw bookmarkError(.permissionDenied, "Requested path is outside the bookmarked folder")
  }

  let resolvedFolderUrl = folderUrl.resolvingSymlinksInPath()
  let normalizedTarget = normalizeIosScopedPath(
    URL(fileURLWithPath: targetPath).resolvingSymlinksInPath().path
  )
  let normalizedFolder = normalizeIosScopedPath(resolvedFolderUrl.path)
  guard resolvedPathIsWithinFolder(normalizedTarget, folder: normalizedFolder) else {
    throw bookmarkError(.permissionDenied, "Resolved path is outside the bookmarked folder")
  }

  if normalizedTarget == normalizedFolder {
    return folderUrl
  }

  let relativePath = String(normalizedTarget.dropFirst(normalizedFolder.count + 1))
  let relativeComponents = relativePath.split(separator: "/").map(String.init)

  return relativeComponents.reduce(folderUrl) { partialUrl, component in
    partialUrl.appendingPathComponent(component)
  }
}