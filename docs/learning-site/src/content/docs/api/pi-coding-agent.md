---
title: pi-coding-agent API 参考
---


- [shouldUseWindowsShell](#gear-shouldusewindowsshell)
- [waitForChildProcess](#gear-waitforchildprocess)
- [detectInstallMethod](#gear-detectinstallmethod)
- [getSelfUpdateCommand](#gear-getselfupdatecommand)
- [getSelfUpdateUnavailableInstruction](#gear-getselfupdateunavailableinstruction)
- [getUpdateInstruction](#gear-getupdateinstruction)
- [getPackageDir](#gear-getpackagedir)
- [getThemesDir](#gear-getthemesdir)
- [getExportTemplateDir](#gear-getexporttemplatedir)
- [getPackageJsonPath](#gear-getpackagejsonpath)
- [getReadmePath](#gear-getreadmepath)
- [getDocsPath](#gear-getdocspath)
- [getExamplesPath](#gear-getexamplespath)
- [getChangelogPath](#gear-getchangelogpath)
- [getInteractiveAssetsDir](#gear-getinteractiveassetsdir)
- [getBundledInteractiveAssetPath](#gear-getbundledinteractiveassetpath)
- [expandTildePath](#gear-expandtildepath)
- [getShareViewerUrl](#gear-getshareviewerurl)
- [getAgentDir](#gear-getagentdir)
- [getCustomThemesDir](#gear-getcustomthemesdir)
- [getModelsPath](#gear-getmodelspath)
- [getAuthPath](#gear-getauthpath)
- [getSettingsPath](#gear-getsettingspath)
- [getToolsDir](#gear-gettoolsdir)
- [getBinDir](#gear-getbindir)
- [getPromptsDir](#gear-getpromptsdir)
- [getSessionsDir](#gear-getsessionsdir)
- [getDebugLogPath](#gear-getdebuglogpath)
- [parseGitUrl](#gear-parsegiturl)
- [canonicalizePath](#gear-canonicalizepath)
- [isLocalPath](#gear-islocalpath)
- [getCwdRelativePath](#gear-getcwdrelativepath)
- [formatPathRelativeToCwdOrAbsolute](#gear-formatpathrelativetocwdorabsolute)
- [takeOverStdout](#gear-takeoverstdout)
- [restoreStdout](#gear-restorestdout)
- [isStdoutTakenOver](#gear-isstdouttakenover)
- [writeRawStdout](#gear-writerawstdout)
- [flushRawStdout](#gear-flushrawstdout)
- [createSourceInfo](#gear-createsourceinfo)
- [createSyntheticSourceInfo](#gear-createsyntheticsourceinfo)
- [closeWatcher](#gear-closewatcher)
- [watchWithErrorHandler](#gear-watchwitherrorhandler)
- [getAvailableThemes](#gear-getavailablethemes)
- [getAvailableThemesWithPaths](#gear-getavailablethemeswithpaths)
- [loadThemeFromPath](#gear-loadthemefrompath)
- [getThemeByName](#gear-getthemebyname)
- [setRegisteredThemes](#gear-setregisteredthemes)
- [initTheme](#gear-inittheme)
- [setTheme](#gear-settheme)
- [setThemeInstance](#gear-setthemeinstance)
- [onThemeChange](#gear-onthemechange)
- [stopThemeWatcher](#gear-stopthemewatcher)
- [getResolvedThemeColors](#gear-getresolvedthemecolors)
- [isLightTheme](#gear-islighttheme)
- [getThemeExportColors](#gear-getthemeexportcolors)
- [highlightCode](#gear-highlightcode)
- [getLanguageFromPath](#gear-getlanguagefrompath)
- [getMarkdownTheme](#gear-getmarkdowntheme)
- [getSelectListTheme](#gear-getselectlisttheme)
- [getEditorTheme](#gear-geteditortheme)
- [getSettingsListTheme](#gear-getsettingslisttheme)
- [getShellConfig](#gear-getshellconfig)
- [getShellEnv](#gear-getshellenv)
- [sanitizeBinaryOutput](#gear-sanitizebinaryoutput)
- [trackDetachedChildPid](#gear-trackdetachedchildpid)
- [untrackDetachedChildPid](#gear-untrackdetachedchildpid)
- [killTrackedDetachedChildren](#gear-killtrackeddetachedchildren)
- [killProcessTree](#gear-killprocesstree)
- [formatKeyText](#gear-formatkeytext)
- [keyText](#gear-keytext)
- [keyDisplayText](#gear-keydisplaytext)
- [keyHint](#gear-keyhint)
- [rawKeyHint](#gear-rawkeyhint)
- [truncateToVisualLines](#gear-truncatetovisuallines)
- [formatSize](#gear-formatsize)
- [truncateHead](#gear-truncatehead)
- [truncateTail](#gear-truncatetail)
- [truncateLine](#gear-truncateline)
- [shortenPath](#gear-shortenpath)
- [str](#gear-str)
- [replaceTabs](#gear-replacetabs)
- [normalizeDisplayText](#gear-normalizedisplaytext)
- [getTextOutput](#gear-gettextoutput)
- [invalidArgText](#gear-invalidargtext)
- [wrapToolDefinition](#gear-wraptooldefinition)
- [wrapToolDefinitions](#gear-wraptooldefinitions)
- [createToolDefinitionFromAgentTool](#gear-createtooldefinitionfromagenttool)
- [createLocalBashOperations](#gear-createlocalbashoperations)
- [createBashToolDefinition](#gear-createbashtooldefinition)
- [createBashTool](#gear-createbashtool)
- [executeBashWithOperations](#gear-executebashwithoperations)
- [bashExecutionToText](#gear-bashexecutiontotext)
- [createBranchSummaryMessage](#gear-createbranchsummarymessage)
- [createCompactionSummaryMessage](#gear-createcompactionsummarymessage)
- [createCustomMessage](#gear-createcustommessage)
- [convertToLlm](#gear-converttollm)
- [migrateSessionEntries](#gear-migratesessionentries)
- [parseSessionEntries](#gear-parsesessionentries)
- [getLatestCompactionEntry](#gear-getlatestcompactionentry)
- [buildSessionContext](#gear-buildsessioncontext)
- [getDefaultSessionDir](#gear-getdefaultsessiondir)
- [loadEntriesFromFile](#gear-loadentriesfromfile)
- [findMostRecentSession](#gear-findmostrecentsession)
- [createFileOps](#gear-createfileops)
- [extractFileOpsFromMessage](#gear-extractfileopsfrommessage)
- [computeFileLists](#gear-computefilelists)
- [formatFileOperations](#gear-formatfileoperations)
- [serializeConversation](#gear-serializeconversation)
- [calculateContextTokens](#gear-calculatecontexttokens)
- [getLastAssistantUsage](#gear-getlastassistantusage)
- [estimateContextTokens](#gear-estimatecontexttokens)
- [shouldCompact](#gear-shouldcompact)
- [estimateTokens](#gear-estimatetokens)
- [findTurnStartIndex](#gear-findturnstartindex)
- [findCutPoint](#gear-findcutpoint)
- [generateSummary](#gear-generatesummary)
- [prepareCompaction](#gear-preparecompaction)
- [compact](#gear-compact)
- [collectEntriesForBranchSummary](#gear-collectentriesforbranchsummary)
- [prepareBranchEntries](#gear-preparebranchentries)
- [generateBranchSummary](#gear-generatebranchsummary)
- [createEventBus](#gear-createeventbus)
- [execCommand](#gear-execcommand)
- [migrateKeybindingsConfig](#gear-migratekeybindingsconfig)
- [resolveConfigValue](#gear-resolveconfigvalue)
- [resolveConfigValueUncached](#gear-resolveconfigvalueuncached)
- [resolveConfigValueOrThrow](#gear-resolveconfigvalueorthrow)
- [resolveHeaders](#gear-resolveheaders)
- [resolveHeadersOrThrow](#gear-resolveheadersorthrow)
- [clearConfigValueCache](#gear-clearconfigvaluecache)
- [parseFrontmatter](#gear-parsefrontmatter)
- [stripFrontmatter](#gear-stripfrontmatter)
- [loadSkillsFromDir](#gear-loadskillsfromdir)
- [formatSkillsForPrompt](#gear-formatskillsforprompt)
- [loadSkills](#gear-loadskills)
- [buildSystemPrompt](#gear-buildsystemprompt)
- [renderDiff](#gear-renderdiff)
- [expandPath](#gear-expandpath)
- [resolveToCwd](#gear-resolvetocwd)
- [resolveReadPath](#gear-resolvereadpath)
- [detectLineEnding](#gear-detectlineending)
- [normalizeToLF](#gear-normalizetolf)
- [restoreLineEndings](#gear-restorelineendings)
- [normalizeForFuzzyMatch](#gear-normalizeforfuzzymatch)
- [fuzzyFindText](#gear-fuzzyfindtext)
- [stripBom](#gear-stripbom)
- [applyEditsToNormalizedContent](#gear-applyeditstonormalizedcontent)
- [generateDiffString](#gear-generatediffstring)
- [computeEditsDiff](#gear-computeeditsdiff)
- [computeEditDiff](#gear-computeeditdiff)
- [withFileMutationQueue](#gear-withfilemutationqueue)
- [createEditToolDefinition](#gear-createedittooldefinition)
- [createEditTool](#gear-createedittool)
- [getToolPath](#gear-gettoolpath)
- [ensureTool](#gear-ensuretool)
- [createFindToolDefinition](#gear-createfindtooldefinition)
- [createFindTool](#gear-createfindtool)
- [createGrepToolDefinition](#gear-creategreptooldefinition)
- [createGrepTool](#gear-creategreptool)
- [createLsToolDefinition](#gear-createlstooldefinition)
- [createLsTool](#gear-createlstool)
- [loadPhoton](#gear-loadphoton)
- [applyExifOrientation](#gear-applyexiforientation)
- [resizeImage](#gear-resizeimage)
- [formatDimensionNote](#gear-formatdimensionnote)
- [detectSupportedImageMimeTypeFromFile](#gear-detectsupportedimagemimetypefromfile)
- [createReadToolDefinition](#gear-createreadtooldefinition)
- [createReadTool](#gear-createreadtool)
- [createWriteToolDefinition](#gear-createwritetooldefinition)
- [createWriteTool](#gear-createwritetool)
- [createToolDefinition](#gear-createtooldefinition)
- [createTool](#gear-createtool)
- [createCodingToolDefinitions](#gear-createcodingtooldefinitions)
- [createReadOnlyToolDefinitions](#gear-createreadonlytooldefinitions)
- [createAllToolDefinitions](#gear-createalltooldefinitions)
- [createCodingTools](#gear-createcodingtools)
- [createReadOnlyTools](#gear-createreadonlytools)
- [createAllTools](#gear-createalltools)
- [defineTool](#gear-definetool)
- [isBashToolResult](#gear-isbashtoolresult)
- [isReadToolResult](#gear-isreadtoolresult)
- [isEditToolResult](#gear-isedittoolresult)
- [isWriteToolResult](#gear-iswritetoolresult)
- [isGrepToolResult](#gear-isgreptoolresult)
- [isFindToolResult](#gear-isfindtoolresult)
- [isLsToolResult](#gear-islstoolresult)
- [isToolCallEventType](#gear-istoolcalleventtype)
- [isToolCallEventType](#gear-istoolcalleventtype)
- [isToolCallEventType](#gear-istoolcalleventtype)
- [isToolCallEventType](#gear-istoolcalleventtype)
- [isToolCallEventType](#gear-istoolcalleventtype)
- [isToolCallEventType](#gear-istoolcalleventtype)
- [isToolCallEventType](#gear-istoolcalleventtype)
- [isToolCallEventType](#gear-istoolcalleventtype)
- [isToolCallEventType](#gear-istoolcalleventtype)
- [isValidThinkingLevel](#gear-isvalidthinkinglevel)
- [parseArgs](#gear-parseargs)
- [printHelp](#gear-printhelp)
- [processFileArguments](#gear-processfilearguments)
- [buildInitialMessage](#gear-buildinitialmessage)
- [getProviderLoginHelp](#gear-getproviderloginhelp)
- [formatNoModelsAvailableMessage](#gear-formatnomodelsavailablemessage)
- [formatNoModelSelectedMessage](#gear-formatnomodelselectedmessage)
- [formatNoApiKeyFoundMessage](#gear-formatnoapikeyfoundmessage)
- [listModels](#gear-listmodels)
- [hasSessionName](#gear-hassessionname)
- [parseSearchQuery](#gear-parsesearchquery)
- [matchSession](#gear-matchsession)
- [filterAndSortSessions](#gear-filterandsortsessions)
- [selectSession](#gear-selectsession)
- [sleep](#gear-sleep)
- [exportSessionToHtml](#gear-exportsessiontohtml)
- [exportFromFile](#gear-exportfromfile)
- [ansiToHtml](#gear-ansitohtml)
- [ansiLinesToHtml](#gear-ansilinestohtml)
- [createToolHtmlRenderer](#gear-createtoolhtmlrenderer)
- [parseCommandArgs](#gear-parsecommandargs)
- [substituteArgs](#gear-substituteargs)
- [loadPromptTemplates](#gear-loadprompttemplates)
- [expandPromptTemplate](#gear-expandprompttemplate)
- [loadProjectContextFiles](#gear-loadprojectcontextfiles)
- [findExactModelReferenceMatch](#gear-findexactmodelreferencematch)
- [parseModelPattern](#gear-parsemodelpattern)
- [resolveModelScope](#gear-resolvemodelscope)
- [resolveCliModel](#gear-resolveclimodel)
- [findInitialModel](#gear-findinitialmodel)
- [restoreModelFromSession](#gear-restoremodelfromsession)
- [isInstallTelemetryEnabled](#gear-isinstalltelemetryenabled)
- [resetTimings](#gear-resettimings)
- [time](#gear-time)
- [printTimings](#gear-printtimings)
- [createAgentSession](#gear-createagentsession)
- [getMissingSessionCwdIssue](#gear-getmissingsessioncwdissue)
- [formatMissingSessionCwdError](#gear-formatmissingsessioncwderror)
- [formatMissingSessionCwdPrompt](#gear-formatmissingsessioncwdprompt)
- [assertSessionCwdExists](#gear-assertsessioncwdexists)
- [parseChangelog](#gear-parsechangelog)
- [compareVersions](#gear-compareversions)
- [getNewEntries](#gear-getnewentries)
- [isWaylandSession](#gear-iswaylandsession)
- [extensionForImageMimeType](#gear-extensionforimagemimetype)
- [readClipboardImage](#gear-readclipboardimage)
- [copyToClipboard](#gear-copytoclipboard)
- [getPiUserAgent](#gear-getpiuseragent)
- [comparePackageVersions](#gear-comparepackageversions)
- [isNewerPackageVersion](#gear-isnewerpackageversion)
- [getLatestPiRelease](#gear-getlatestpirelease)
- [getLatestPiVersion](#gear-getlatestpiversion)
- [checkForNewPiVersion](#gear-checkfornewpiversion)
- [convertToPng](#gear-converttopng)
- [isApiKeyLoginProvider](#gear-isapikeyloginprovider)
- [runPrintMode](#gear-runprintmode)
- [serializeJsonLine](#gear-serializejsonline)
- [attachJsonlLineReader](#gear-attachjsonllinereader)
- [runRpcMode](#gear-runrpcmode)
- [createExtensionRuntime](#gear-createextensionruntime)
- [loadExtensionFromFactory](#gear-loadextensionfromfactory)
- [loadExtensions](#gear-loadextensions)
- [discoverAndLoadExtensions](#gear-discoverandloadextensions)
- [emitSessionShutdownEvent](#gear-emitsessionshutdownevent)
- [wrapRegisteredTool](#gear-wrapregisteredtool)
- [wrapRegisteredTools](#gear-wrapregisteredtools)
- [parseSkillBlock](#gear-parseskillblock)
- [createAgentSessionServices](#gear-createagentsessionservices)
- [createAgentSessionFromServices](#gear-createagentsessionfromservices)
- [createAgentSessionRuntime](#gear-createagentsessionruntime)
- [migrateAuthToAuthJson](#gear-migrateauthtoauthjson)
- [migrateSessionsFromAgentRoot](#gear-migratesessionsfromagentroot)
- [showDeprecationWarnings](#gear-showdeprecationwarnings)
- [runMigrations](#gear-runmigrations)
- [selectConfig](#gear-selectconfig)
- [handleConfigCommand](#gear-handleconfigcommand)
- [handlePackageCommand](#gear-handlepackagecommand)
- [main](#gear-main)

### :gear: shouldUseWindowsShell

| Function | Type |
| ---------- | ---------- |
| `shouldUseWindowsShell` | `(command: string) => boolean` |

### :gear: waitForChildProcess

Wait for a child process to terminate without hanging on inherited stdio handles.

On Windows, daemonized descendants can inherit the child's stdout/stderr pipe
handles. In that case the child emits `exit`, but `close` can hang forever even
though the original process is already gone. We wait briefly for stdio to end,
then forcibly stop tracking the inherited handles.

| Function | Type |
| ---------- | ---------- |
| `waitForChildProcess` | `(child: ChildProcess) => Promise<number or null>` |

### :gear: detectInstallMethod

检测当前安装方式。
通过分析模块路径和可执行文件路径中是否包含包管理器特征字符串来判断。
Bun 编译二进制优先级最高（直接返回），其余按 pnpm → yarn → bun → npm 的顺序匹配。

| Function | Type |
| ---------- | ---------- |
| `detectInstallMethod` | `() => InstallMethod` |

### :gear: getSelfUpdateCommand

获取自更新命令。
仅在同时满足以下条件时返回命令：
1. 能检测到安装方式
2. 安装由全局包管理器管理（非本地项目安装）
3. 包安装目录具有写入权限

| Function | Type |
| ---------- | ---------- |
| `getSelfUpdateCommand` | `(packageName: string, npmCommand?: string[] or undefined, updatePackageName?: string) => SelfUpdateCommand or undefined` |

Parameters:

* `packageName`: - 当前已安装的包名
* `npmCommand`: - 自定义 npm 命令（覆盖默认的 npm 行为）
* `updatePackageName`: - 要更新到的目标包名（用于包名迁移场景）


### :gear: getSelfUpdateUnavailableInstruction

获取自更新不可用时的说明信息。
根据不可用的原因返回不同的提示：
- Bun 编译二进制 → 提示从 GitHub Releases 下载
- 路径不可写 → 提示手动使用包管理器更新
- 非全局安装 → 提示使用原始安装方式进行更新

| Function | Type |
| ---------- | ---------- |
| `getSelfUpdateUnavailableInstruction` | `(packageName: string, npmCommand?: string[] or undefined, updatePackageName?: string) => string` |

Parameters:

* `packageName`: - 当前已安装的包名
* `npmCommand`: - 自定义 npm 命令
* `updatePackageName`: - 要更新到的目标包名


### :gear: getUpdateInstruction

获取更新提示信息。
如果自更新可用则返回具体命令，否则返回不可用说明。
用于向用户展示如何将应用更新到最新版本。

| Function | Type |
| ---------- | ---------- |
| `getUpdateInstruction` | `(packageName: string) => string` |

Parameters:

* `packageName`: - 包名


### :gear: getPackageDir

获取包资产的基础目录（用于定位主题、package.json、README.md、CHANGELOG.md 等）。
- Bun 编译二进制：返回可执行文件所在目录
- Node.js（dist/ 模式）：返回 __dirname（即 dist/ 目录）
- tsx（src/ 模式）：返回父目录（即包根目录，因为 __dirname 是 src/）

支持通过环境变量 `PI_PACKAGE_DIR` 覆盖（适用于 Nix/Guix 等路径特殊的包管理器）。

| Function | Type |
| ---------- | ---------- |
| `getPackageDir` | `() => string` |

### :gear: getThemesDir

获取内置主题目录路径。
- Bun 编译二进制：可执行文件旁的 theme/ 目录
- Node.js（dist/）：dist/modes/interactive/theme/
- tsx（src/）：src/modes/interactive/theme/

| Function | Type |
| ---------- | ---------- |
| `getThemesDir` | `() => string` |

### :gear: getExportTemplateDir

获取 HTML 导出模板目录路径。
- Bun 编译二进制：可执行文件旁的 export-html/ 目录
- Node.js（dist/）：dist/core/export-html/
- tsx（src/）：src/core/export-html/

| Function | Type |
| ---------- | ---------- |
| `getExportTemplateDir` | `() => string` |

### :gear: getPackageJsonPath

获取 package.json 的路径

| Function | Type |
| ---------- | ---------- |
| `getPackageJsonPath` | `() => string` |

### :gear: getReadmePath

获取 README.md 的路径

| Function | Type |
| ---------- | ---------- |
| `getReadmePath` | `() => string` |

### :gear: getDocsPath

获取文档目录的路径

| Function | Type |
| ---------- | ---------- |
| `getDocsPath` | `() => string` |

### :gear: getExamplesPath

获取示例目录的路径

| Function | Type |
| ---------- | ---------- |
| `getExamplesPath` | `() => string` |

### :gear: getChangelogPath

获取 CHANGELOG.md 的路径

| Function | Type |
| ---------- | ---------- |
| `getChangelogPath` | `() => string` |

### :gear: getInteractiveAssetsDir

获取内置交互模式资产目录路径。
- Bun 编译二进制：可执行文件旁的 assets/ 目录
- Node.js（dist/）：dist/modes/interactive/assets/
- tsx（src/）：src/modes/interactive/assets/

| Function | Type |
| ---------- | ---------- |
| `getInteractiveAssetsDir` | `() => string` |

### :gear: getBundledInteractiveAssetPath

获取指定名称的捆绑交互模式资产文件的完整路径。

| Function | Type |
| ---------- | ---------- |
| `getBundledInteractiveAssetPath` | `(name: string) => string` |

Parameters:

* `name`: - 资产文件名


### :gear: expandTildePath

展开路径中的波浪号（~）为用户主目录。
支持 `~`（主目录本身）和 `~/path`（主目录下的子路径）两种形式。

| Function | Type |
| ---------- | ---------- |
| `expandTildePath` | `(path: string) => string` |

Parameters:

* `path`: - 可能包含 ~ 前缀的路径


### :gear: getShareViewerUrl

获取会话分享查看器的 URL。
可通过环境变量 `PI_SHARE_VIEWER_URL` 覆盖默认地址。

| Function | Type |
| ---------- | ---------- |
| `getShareViewerUrl` | `(gistId: string) => string` |

Parameters:

* `gistId`: - GitHub Gist ID，作为 URL 锚点附加到查看器地址后


### :gear: getAgentDir

获取代理配置目录路径（如 ~/.pi/agent/）。
支持通过环境变量（如 `PI_CODING_AGENT_DIR`）覆盖默认位置。
认证文件、模型配置、会话存储等均位于此目录下。

| Function | Type |
| ---------- | ---------- |
| `getAgentDir` | `() => string` |

### :gear: getCustomThemesDir

获取用户自定义主题目录路径

| Function | Type |
| ---------- | ---------- |
| `getCustomThemesDir` | `() => string` |

### :gear: getModelsPath

获取模型配置文件（models.json）路径，用于自定义模型列表和参数

| Function | Type |
| ---------- | ---------- |
| `getModelsPath` | `() => string` |

### :gear: getAuthPath

获取认证文件（auth.json）路径，存储各提供商的 OAuth 凭证

| Function | Type |
| ---------- | ---------- |
| `getAuthPath` | `() => string` |

### :gear: getSettingsPath

获取用户设置文件（settings.json）路径

| Function | Type |
| ---------- | ---------- |
| `getSettingsPath` | `() => string` |

### :gear: getToolsDir

获取自定义工具目录路径

| Function | Type |
| ---------- | ---------- |
| `getToolsDir` | `() => string` |

### :gear: getBinDir

获取托管二进制文件目录路径（用于存放 fd、rg 等辅助工具的可执行文件）

| Function | Type |
| ---------- | ---------- |
| `getBinDir` | `() => string` |

### :gear: getPromptsDir

获取提示模板目录路径

| Function | Type |
| ---------- | ---------- |
| `getPromptsDir` | `() => string` |

### :gear: getSessionsDir

获取会话存储目录路径

| Function | Type |
| ---------- | ---------- |
| `getSessionsDir` | `() => string` |

### :gear: getDebugLogPath

获取调试日志文件路径

| Function | Type |
| ---------- | ---------- |
| `getDebugLogPath` | `() => string` |

### :gear: parseGitUrl

Parse git source into a GitSource.

Rules:
- With git: prefix, accept all historical shorthand forms.
- Without git: prefix, only accept explicit protocol URLs.

| Function | Type |
| ---------- | ---------- |
| `parseGitUrl` | `(source: string) => GitSource or null` |

### :gear: canonicalizePath

Resolve a path to its canonical (real) form, following symlinks.
Falls back to the raw path if resolution fails (e.g. the target does
not exist yet), so that callers never crash on missing filesystem
entries.

| Function | Type |
| ---------- | ---------- |
| `canonicalizePath` | `(path: string) => string` |

### :gear: isLocalPath

Returns true if the value is NOT a package source (npm:, git:, etc.)
or a URL protocol. Bare names and relative paths without ./ prefix
are considered local.

| Function | Type |
| ---------- | ---------- |
| `isLocalPath` | `(value: string) => boolean` |

### :gear: getCwdRelativePath

| Function | Type |
| ---------- | ---------- |
| `getCwdRelativePath` | `(filePath: string, cwd: string) => string or undefined` |

### :gear: formatPathRelativeToCwdOrAbsolute

| Function | Type |
| ---------- | ---------- |
| `formatPathRelativeToCwdOrAbsolute` | `(filePath: string, cwd: string) => string` |

### :gear: takeOverStdout

| Function | Type |
| ---------- | ---------- |
| `takeOverStdout` | `() => void` |

### :gear: restoreStdout

| Function | Type |
| ---------- | ---------- |
| `restoreStdout` | `() => void` |

### :gear: isStdoutTakenOver

| Function | Type |
| ---------- | ---------- |
| `isStdoutTakenOver` | `() => boolean` |

### :gear: writeRawStdout

| Function | Type |
| ---------- | ---------- |
| `writeRawStdout` | `(text: string) => void` |

### :gear: flushRawStdout

| Function | Type |
| ---------- | ---------- |
| `flushRawStdout` | `() => Promise<void>` |

### :gear: createSourceInfo

| Function | Type |
| ---------- | ---------- |
| `createSourceInfo` | `(path: string, metadata: PathMetadata) => SourceInfo` |

### :gear: createSyntheticSourceInfo

| Function | Type |
| ---------- | ---------- |
| `createSyntheticSourceInfo` | `(path: string, options: { source: string; scope?: SourceScope or undefined; origin?: SourceOrigin or undefined; baseDir?: string or undefined; }) => SourceInfo` |

### :gear: closeWatcher

| Function | Type |
| ---------- | ---------- |
| `closeWatcher` | `(watcher: FSWatcher or null or undefined) => void` |

### :gear: watchWithErrorHandler

| Function | Type |
| ---------- | ---------- |
| `watchWithErrorHandler` | `(path: string, listener: WatchListener<string>, onError: () => void) => FSWatcher or null` |

### :gear: getAvailableThemes

| Function | Type |
| ---------- | ---------- |
| `getAvailableThemes` | `() => string[]` |

### :gear: getAvailableThemesWithPaths

| Function | Type |
| ---------- | ---------- |
| `getAvailableThemesWithPaths` | `() => ThemeInfo[]` |

### :gear: loadThemeFromPath

| Function | Type |
| ---------- | ---------- |
| `loadThemeFromPath` | `(themePath: string, mode?: ColorMode or undefined) => Theme` |

### :gear: getThemeByName

| Function | Type |
| ---------- | ---------- |
| `getThemeByName` | `(name: string) => Theme or undefined` |

### :gear: setRegisteredThemes

| Function | Type |
| ---------- | ---------- |
| `setRegisteredThemes` | `(themes: Theme[]) => void` |

### :gear: initTheme

| Function | Type |
| ---------- | ---------- |
| `initTheme` | `(themeName?: string or undefined, enableWatcher?: boolean) => void` |

### :gear: setTheme

| Function | Type |
| ---------- | ---------- |
| `setTheme` | `(name: string, enableWatcher?: boolean) => { success: boolean; error?: string or undefined; }` |

### :gear: setThemeInstance

| Function | Type |
| ---------- | ---------- |
| `setThemeInstance` | `(themeInstance: Theme) => void` |

### :gear: onThemeChange

| Function | Type |
| ---------- | ---------- |
| `onThemeChange` | `(callback: () => void) => void` |

### :gear: stopThemeWatcher

| Function | Type |
| ---------- | ---------- |
| `stopThemeWatcher` | `() => void` |

### :gear: getResolvedThemeColors

Get resolved theme colors as CSS-compatible hex strings.
Used by HTML export to generate CSS custom properties.

| Function | Type |
| ---------- | ---------- |
| `getResolvedThemeColors` | `(themeName?: string or undefined) => Record<string, string>` |

### :gear: isLightTheme

Check if a theme is a "light" theme (for CSS that needs light/dark variants).

| Function | Type |
| ---------- | ---------- |
| `isLightTheme` | `(themeName?: string or undefined) => boolean` |

### :gear: getThemeExportColors

Get explicit export colors from theme JSON, if specified.
Returns undefined for each color that isn't explicitly set.

| Function | Type |
| ---------- | ---------- |
| `getThemeExportColors` | `(themeName?: string or undefined) => { pageBg?: string or undefined; cardBg?: string or undefined; infoBg?: string or undefined; }` |

### :gear: highlightCode

Highlight code with syntax coloring based on file extension or language.
Returns array of highlighted lines.

| Function | Type |
| ---------- | ---------- |
| `highlightCode` | `(code: string, lang?: string or undefined) => string[]` |

### :gear: getLanguageFromPath

Get language identifier from file path extension.

| Function | Type |
| ---------- | ---------- |
| `getLanguageFromPath` | `(filePath: string) => string or undefined` |

### :gear: getMarkdownTheme

| Function | Type |
| ---------- | ---------- |
| `getMarkdownTheme` | `() => MarkdownTheme` |

### :gear: getSelectListTheme

| Function | Type |
| ---------- | ---------- |
| `getSelectListTheme` | `() => SelectListTheme` |

### :gear: getEditorTheme

| Function | Type |
| ---------- | ---------- |
| `getEditorTheme` | `() => EditorTheme` |

### :gear: getSettingsListTheme

| Function | Type |
| ---------- | ---------- |
| `getSettingsListTheme` | `() => SettingsListTheme` |

### :gear: getShellConfig

Resolve shell configuration based on platform and an optional explicit shell path.
Resolution order:
1. User-specified shellPath
2. On Windows: Git Bash in known locations, then bash on PATH
3. On Unix: /bin/bash, then bash on PATH, then fallback to sh

| Function | Type |
| ---------- | ---------- |
| `getShellConfig` | `(customShellPath?: string or undefined) => ShellConfig` |

### :gear: getShellEnv

| Function | Type |
| ---------- | ---------- |
| `getShellEnv` | `() => ProcessEnv` |

### :gear: sanitizeBinaryOutput

Sanitize binary output for display/storage.
Removes characters that crash string-width or cause display issues:
- Control characters (except tab, newline, carriage return)
- Lone surrogates
- Unicode Format characters (crash string-width due to a bug)
- Characters with undefined code points

| Function | Type |
| ---------- | ---------- |
| `sanitizeBinaryOutput` | `(str: string) => string` |

### :gear: trackDetachedChildPid

| Function | Type |
| ---------- | ---------- |
| `trackDetachedChildPid` | `(pid: number) => void` |

### :gear: untrackDetachedChildPid

| Function | Type |
| ---------- | ---------- |
| `untrackDetachedChildPid` | `(pid: number) => void` |

### :gear: killTrackedDetachedChildren

| Function | Type |
| ---------- | ---------- |
| `killTrackedDetachedChildren` | `() => void` |

### :gear: killProcessTree

Kill a process and all its children (cross-platform)

| Function | Type |
| ---------- | ---------- |
| `killProcessTree` | `(pid: number) => void` |

### :gear: formatKeyText

| Function | Type |
| ---------- | ---------- |
| `formatKeyText` | `(key: string, options?: KeyTextFormatOptions) => string` |

### :gear: keyText

| Function | Type |
| ---------- | ---------- |
| `keyText` | `(keybinding: keyof Keybindings) => string` |

### :gear: keyDisplayText

| Function | Type |
| ---------- | ---------- |
| `keyDisplayText` | `(keybinding: keyof Keybindings) => string` |

### :gear: keyHint

| Function | Type |
| ---------- | ---------- |
| `keyHint` | `(keybinding: keyof Keybindings, description: string) => string` |

### :gear: rawKeyHint

| Function | Type |
| ---------- | ---------- |
| `rawKeyHint` | `(key: string, description: string) => string` |

### :gear: truncateToVisualLines

Truncate text to a maximum number of visual lines (from the end).
This accounts for line wrapping based on terminal width.

| Function | Type |
| ---------- | ---------- |
| `truncateToVisualLines` | `(text: string, maxVisualLines: number, width: number, paddingX?: number) => VisualTruncateResult` |

Parameters:

* `text`: - The text content (may contain newlines)
* `maxVisualLines`: - Maximum number of visual lines to show
* `width`: - Terminal/render width
* `paddingX`: - Horizontal padding for Text component (default 0).
  Use 0 when result will be placed in a Box (Box adds its own padding).
  Use 1 when result will be placed in a plain Container.


Returns:

The truncated visual lines and count of skipped lines

### :gear: formatSize

Format bytes as human-readable size.

| Function | Type |
| ---------- | ---------- |
| `formatSize` | `(bytes: number) => string` |

### :gear: truncateHead

Truncate content from the head (keep first N lines/bytes).
Suitable for file reads where you want to see the beginning.

Never returns partial lines. If first line exceeds byte limit,
returns empty content with firstLineExceedsLimit=true.

| Function | Type |
| ---------- | ---------- |
| `truncateHead` | `(content: string, options?: TruncationOptions) => TruncationResult` |

### :gear: truncateTail

Truncate content from the tail (keep last N lines/bytes).
Suitable for bash output where you want to see the end (errors, final results).

May return partial first line if the last line of original content exceeds byte limit.

| Function | Type |
| ---------- | ---------- |
| `truncateTail` | `(content: string, options?: TruncationOptions) => TruncationResult` |

### :gear: truncateLine

Truncate a single line to max characters, adding [truncated] suffix.
Used for grep match lines.

| Function | Type |
| ---------- | ---------- |
| `truncateLine` | `(line: string, maxChars?: number) => { text: string; wasTruncated: boolean; }` |

### :gear: shortenPath

| Function | Type |
| ---------- | ---------- |
| `shortenPath` | `(path: unknown) => string` |

### :gear: str

| Function | Type |
| ---------- | ---------- |
| `str` | `(value: unknown) => string or null` |

### :gear: replaceTabs

| Function | Type |
| ---------- | ---------- |
| `replaceTabs` | `(text: string) => string` |

### :gear: normalizeDisplayText

| Function | Type |
| ---------- | ---------- |
| `normalizeDisplayText` | `(text: string) => string` |

### :gear: getTextOutput

| Function | Type |
| ---------- | ---------- |
| `getTextOutput` | `(result: { content: { type: string; text?: string or undefined; data?: string or undefined; mimeType?: string or undefined; }[]; } or undefined, showImages: boolean) => string` |

### :gear: invalidArgText

| Function | Type |
| ---------- | ---------- |
| `invalidArgText` | `(theme: { fg: (name: any, text: string) => string; }) => string` |

### :gear: wrapToolDefinition

Wrap a ToolDefinition into an AgentTool for the core runtime.

| Function | Type |
| ---------- | ---------- |
| `wrapToolDefinition` | `<TDetails = unknown>(definition: ToolDefinition<any, TDetails, any>, ctxFactory?: (() => ExtensionContext) or undefined) => AgentTool<any, TDetails>` |

### :gear: wrapToolDefinitions

Wrap multiple ToolDefinitions into AgentTools for the core runtime.

| Function | Type |
| ---------- | ---------- |
| `wrapToolDefinitions` | `(definitions: ToolDefinition<any, any, any>[], ctxFactory?: (() => ExtensionContext) or undefined) => AgentTool<any, any>[]` |

### :gear: createToolDefinitionFromAgentTool

Synthesize a minimal ToolDefinition from an AgentTool.

This keeps AgentSession's internal registry definition-first even when a caller
provides plain AgentTool overrides that do not include prompt metadata or renderers.

| Function | Type |
| ---------- | ---------- |
| `createToolDefinitionFromAgentTool` | `(tool: AgentTool<any, any>) => ToolDefinition<any, unknown, any>` |

### :gear: createLocalBashOperations

创建本地 shell 执行后端。
可用于 user_bash 事件拦截后仍需 pi 标准 shell 行为的场景。

| Function | Type |
| ---------- | ---------- |
| `createLocalBashOperations` | `(options?: { shellPath?: string or undefined; } or undefined) => BashOperations` |

### :gear: createBashToolDefinition

| Function | Type |
| ---------- | ---------- |
| `createBashToolDefinition` | `(cwd: string, options?: BashToolOptions or undefined) => ToolDefinition<TObject<{ command: TString; timeout: TOptional<TNumber>; }>, BashToolDetails or undefined, BashRenderState>` |

### :gear: createBashTool

| Function | Type |
| ---------- | ---------- |
| `createBashTool` | `(cwd: string, options?: BashToolOptions or undefined) => AgentTool<TObject<{ command: TString; timeout: TOptional<TNumber>; }>, any>` |

### :gear: executeBashWithOperations

Execute a bash command using custom BashOperations.
Used for remote execution (SSH, containers, etc.).

| Function | Type |
| ---------- | ---------- |
| `executeBashWithOperations` | `(command: string, cwd: string, operations: BashOperations, options?: BashExecutorOptions or undefined) => Promise<BashResult>` |

### :gear: bashExecutionToText

Convert a BashExecutionMessage to user message text for LLM context.

| Function | Type |
| ---------- | ---------- |
| `bashExecutionToText` | `(msg: BashExecutionMessage) => string` |

### :gear: createBranchSummaryMessage

| Function | Type |
| ---------- | ---------- |
| `createBranchSummaryMessage` | `(summary: string, fromId: string, timestamp: string) => BranchSummaryMessage` |

### :gear: createCompactionSummaryMessage

| Function | Type |
| ---------- | ---------- |
| `createCompactionSummaryMessage` | `(summary: string, tokensBefore: number, timestamp: string) => CompactionSummaryMessage` |

### :gear: createCustomMessage

Convert CustomMessageEntry to AgentMessage format

| Function | Type |
| ---------- | ---------- |
| `createCustomMessage` | `(customType: string, content: string or (TextContent or ImageContent)[], display: boolean, details: unknown, timestamp: string) => CustomMessage<unknown>` |

### :gear: convertToLlm

Transform AgentMessages (including custom types) to LLM-compatible Messages.

This is used by:
- Agent's transormToLlm option (for prompt calls and queued messages)
- Compaction's generateSummary (for summarization)
- Custom extensions and tools

| Function | Type |
| ---------- | ---------- |
| `convertToLlm` | `(messages: AgentMessage[]) => Message[]` |

### :gear: migrateSessionEntries

Exported for testing

| Function | Type |
| ---------- | ---------- |
| `migrateSessionEntries` | `(entries: FileEntry[]) => void` |

### :gear: parseSessionEntries

Exported for compaction.test.ts

| Function | Type |
| ---------- | ---------- |
| `parseSessionEntries` | `(content: string) => FileEntry[]` |

### :gear: getLatestCompactionEntry

| Function | Type |
| ---------- | ---------- |
| `getLatestCompactionEntry` | `(entries: SessionEntry[]) => CompactionEntry<unknown> or null` |

### :gear: buildSessionContext

Build the session context from entries using tree traversal.
If leafId is provided, walks from that entry to root.
Handles compaction and branch summaries along the path.

| Function | Type |
| ---------- | ---------- |
| `buildSessionContext` | `(entries: SessionEntry[], leafId?: string or null or undefined, byId?: Map<string, SessionEntry> or undefined) => SessionContext` |

### :gear: getDefaultSessionDir

Compute the default session directory for a cwd.
Encodes cwd into a safe directory name under ~/.pi/agent/sessions/.

| Function | Type |
| ---------- | ---------- |
| `getDefaultSessionDir` | `(cwd: string, agentDir?: string) => string` |

### :gear: loadEntriesFromFile

Exported for testing

| Function | Type |
| ---------- | ---------- |
| `loadEntriesFromFile` | `(filePath: string) => FileEntry[]` |

### :gear: findMostRecentSession

Exported for testing

| Function | Type |
| ---------- | ---------- |
| `findMostRecentSession` | `(sessionDir: string) => string or null` |

### :gear: createFileOps

| Function | Type |
| ---------- | ---------- |
| `createFileOps` | `() => FileOperations` |

### :gear: extractFileOpsFromMessage

Extract file operations from tool calls in an assistant message.

| Function | Type |
| ---------- | ---------- |
| `extractFileOpsFromMessage` | `(message: AgentMessage, fileOps: FileOperations) => void` |

### :gear: computeFileLists

Compute final file lists from file operations.
Returns readFiles (files only read, not modified) and modifiedFiles.

| Function | Type |
| ---------- | ---------- |
| `computeFileLists` | `(fileOps: FileOperations) => { readFiles: string[]; modifiedFiles: string[]; }` |

### :gear: formatFileOperations

Format file operations as XML tags for summary.

| Function | Type |
| ---------- | ---------- |
| `formatFileOperations` | `(readFiles: string[], modifiedFiles: string[]) => string` |

### :gear: serializeConversation

Serialize LLM messages to text for summarization.
This prevents the model from treating it as a conversation to continue.
Call convertToLlm() first to handle custom message types.

Tool results are truncated to keep the summarization request within
reasonable token budgets. Full content is not needed for summarization.

| Function | Type |
| ---------- | ---------- |
| `serializeConversation` | `(messages: Message[]) => string` |

### :gear: calculateContextTokens

Calculate total context tokens from usage.
Uses the native totalTokens field when available, falls back to computing from components.

| Function | Type |
| ---------- | ---------- |
| `calculateContextTokens` | `(usage: Usage) => number` |

### :gear: getLastAssistantUsage

Find the last non-aborted assistant message usage from session entries.

| Function | Type |
| ---------- | ---------- |
| `getLastAssistantUsage` | `(entries: SessionEntry[]) => Usage or undefined` |

### :gear: estimateContextTokens

Estimate context tokens from messages, using the last assistant usage when available.
If there are messages after the last usage, estimate their tokens with estimateTokens.

| Function | Type |
| ---------- | ---------- |
| `estimateContextTokens` | `(messages: AgentMessage[]) => ContextUsageEstimate` |

### :gear: shouldCompact

Check if compaction should trigger based on context usage.

| Function | Type |
| ---------- | ---------- |
| `shouldCompact` | `(contextTokens: number, contextWindow: number, settings: CompactionSettings) => boolean` |

### :gear: estimateTokens

Estimate token count for a message using chars/4 heuristic.
This is conservative (overestimates tokens).

| Function | Type |
| ---------- | ---------- |
| `estimateTokens` | `(message: AgentMessage) => number` |

### :gear: findTurnStartIndex

Find the user message (or bashExecution) that starts the turn containing the given entry index.
Returns -1 if no turn start found before the index.
BashExecutionMessage is treated like a user message for turn boundaries.

| Function | Type |
| ---------- | ---------- |
| `findTurnStartIndex` | `(entries: SessionEntry[], entryIndex: number, startIndex: number) => number` |

### :gear: findCutPoint

Find the cut point in session entries that keeps approximately `keepRecentTokens`.

Algorithm: Walk backwards from newest, accumulating estimated message sizes.
Stop when we've accumulated >= keepRecentTokens. Cut at that point.

Can cut at user OR assistant messages (never tool results). When cutting at an
assistant message with tool calls, its tool results come after and will be kept.

Returns CutPointResult with:
- firstKeptEntryIndex: the entry index to start keeping from
- turnStartIndex: if cutting mid-turn, the user message that started that turn
- isSplitTurn: whether we're cutting in the middle of a turn

Only considers entries between `startIndex` and `endIndex` (exclusive).

| Function | Type |
| ---------- | ---------- |
| `findCutPoint` | `(entries: SessionEntry[], startIndex: number, endIndex: number, keepRecentTokens: number) => CutPointResult` |

### :gear: generateSummary

Generate a summary of the conversation using the LLM.
If previousSummary is provided, uses the update prompt to merge.

| Function | Type |
| ---------- | ---------- |
| `generateSummary` | `(currentMessages: AgentMessage[], model: Model<any>, reserveTokens: number, apiKey: string, headers?: Record<string, string> or undefined, signal?: AbortSignal or undefined, customInstructions?: string or undefined, previousSummary?: string or undefined, thinkingLevel?: ThinkingLevel or undefined) => Promise<...>` |

### :gear: prepareCompaction

| Function | Type |
| ---------- | ---------- |
| `prepareCompaction` | `(pathEntries: SessionEntry[], settings: CompactionSettings) => CompactionPreparation or undefined` |

### :gear: compact

Generate summaries for compaction using prepared data.
Returns CompactionResult - SessionManager adds uuid/parentUuid when saving.

| Function | Type |
| ---------- | ---------- |
| `compact` | `(preparation: CompactionPreparation, model: Model<any>, apiKey: string, headers?: Record<string, string> or undefined, customInstructions?: string or undefined, signal?: AbortSignal or undefined, thinkingLevel?: ThinkingLevel or undefined) => Promise<...>` |

Parameters:

* `preparation`: - Pre-calculated preparation from prepareCompaction()
* `customInstructions`: - Optional custom focus for the summary


### :gear: collectEntriesForBranchSummary

Collect entries that should be summarized when navigating from one position to another.

Walks from oldLeafId back to the common ancestor with targetId, collecting entries
along the way. Does NOT stop at compaction boundaries - those are included and their
summaries become context.

| Function | Type |
| ---------- | ---------- |
| `collectEntriesForBranchSummary` | `(session: ReadonlySessionManager, oldLeafId: string or null, targetId: string) => CollectEntriesResult` |

Parameters:

* `session`: - Session manager (read-only access)
* `oldLeafId`: - Current position (where we're navigating from)
* `targetId`: - Target position (where we're navigating to)


Returns:

Entries to summarize and the common ancestor

### :gear: prepareBranchEntries

Prepare entries for summarization with token budget.

Walks entries from NEWEST to OLDEST, adding messages until we hit the token budget.
This ensures we keep the most recent context when the branch is too long.

Also collects file operations from:
- Tool calls in assistant messages
- Existing branch_summary entries' details (for cumulative tracking)

| Function | Type |
| ---------- | ---------- |
| `prepareBranchEntries` | `(entries: SessionEntry[], tokenBudget?: number) => BranchPreparation` |

Parameters:

* `entries`: - Entries in chronological order
* `tokenBudget`: - Maximum tokens to include (0 = no limit)


### :gear: generateBranchSummary

Generate a summary of abandoned branch entries.

| Function | Type |
| ---------- | ---------- |
| `generateBranchSummary` | `(entries: SessionEntry[], options: GenerateBranchSummaryOptions) => Promise<BranchSummaryResult>` |

Parameters:

* `entries`: - Session entries to summarize (chronological order)
* `options`: - Generation options


### :gear: createEventBus

| Function | Type |
| ---------- | ---------- |
| `createEventBus` | `() => EventBusController` |

### :gear: execCommand

Execute a shell command and return stdout/stderr/code.
Supports timeout and abort signal.

| Function | Type |
| ---------- | ---------- |
| `execCommand` | `(command: string, args: string[], cwd: string, options?: ExecOptions or undefined) => Promise<ExecResult>` |

### :gear: migrateKeybindingsConfig

| Function | Type |
| ---------- | ---------- |
| `migrateKeybindingsConfig` | `(rawConfig: Record<string, unknown>) => { config: Record<string, unknown>; migrated: boolean; }` |

### :gear: resolveConfigValue

Resolve a config value (API key, header value, etc.) to an actual value.
- If starts with "!", executes the rest as a shell command and uses stdout (cached)
- Otherwise checks environment variable first, then treats as literal (not cached)

| Function | Type |
| ---------- | ---------- |
| `resolveConfigValue` | `(config: string) => string or undefined` |

### :gear: resolveConfigValueUncached

Resolve all header values using the same resolution logic as API keys.

| Function | Type |
| ---------- | ---------- |
| `resolveConfigValueUncached` | `(config: string) => string or undefined` |

### :gear: resolveConfigValueOrThrow

| Function | Type |
| ---------- | ---------- |
| `resolveConfigValueOrThrow` | `(config: string, description: string) => string` |

### :gear: resolveHeaders

Resolve all header values using the same resolution logic as API keys.

| Function | Type |
| ---------- | ---------- |
| `resolveHeaders` | `(headers: Record<string, string> or undefined) => Record<string, string> or undefined` |

### :gear: resolveHeadersOrThrow

| Function | Type |
| ---------- | ---------- |
| `resolveHeadersOrThrow` | `(headers: Record<string, string> or undefined, description: string) => Record<string, string> or undefined` |

### :gear: clearConfigValueCache

Clear the config value command cache. Exported for testing.

| Function | Type |
| ---------- | ---------- |
| `clearConfigValueCache` | `() => void` |

### :gear: parseFrontmatter

| Function | Type |
| ---------- | ---------- |
| `parseFrontmatter` | `<T extends Record<string, unknown> = Record<string, unknown>>(content: string) => ParsedFrontmatter<T>` |

### :gear: stripFrontmatter

| Function | Type |
| ---------- | ---------- |
| `stripFrontmatter` | `(content: string) => string` |

### :gear: loadSkillsFromDir

Load skills from a directory.

Discovery rules:
- if a directory contains SKILL.md, treat it as a skill root and do not recurse further
- otherwise, load direct .md children in the root
- recurse into subdirectories to find SKILL.md

| Function | Type |
| ---------- | ---------- |
| `loadSkillsFromDir` | `(options: LoadSkillsFromDirOptions) => LoadSkillsResult` |

### :gear: formatSkillsForPrompt

Format skills for inclusion in a system prompt.
Uses XML format per Agent Skills standard.
See: https://agentskills.io/integrate-skills

Skills with disableModelInvocation=true are excluded from the prompt
(they can only be invoked explicitly via /skill:name commands).

| Function | Type |
| ---------- | ---------- |
| `formatSkillsForPrompt` | `(skills: Skill[]) => string` |

### :gear: loadSkills

Load skills from all configured locations.
Returns skills and any validation diagnostics.

| Function | Type |
| ---------- | ---------- |
| `loadSkills` | `(options: LoadSkillsOptions) => LoadSkillsResult` |

### :gear: buildSystemPrompt

构建包含工具列表、行为准则和项目上下文的完整系统提示

| Function | Type |
| ---------- | ---------- |
| `buildSystemPrompt` | `(options: BuildSystemPromptOptions) => string` |

Parameters:

* `options`: - 系统提示构建选项


Returns:

完整的系统提示字符串

### :gear: renderDiff

Render a diff string with colored lines and intra-line change highlighting.
- Context lines: dim/gray
- Removed lines: red, with inverse on changed tokens
- Added lines: green, with inverse on changed tokens

| Function | Type |
| ---------- | ---------- |
| `renderDiff` | `(diffText: string, _options?: RenderDiffOptions) => string` |

### :gear: expandPath

| Function | Type |
| ---------- | ---------- |
| `expandPath` | `(filePath: string) => string` |

### :gear: resolveToCwd

Resolve a path relative to the given cwd.
Handles ~ expansion and absolute paths.

| Function | Type |
| ---------- | ---------- |
| `resolveToCwd` | `(filePath: string, cwd: string) => string` |

### :gear: resolveReadPath

| Function | Type |
| ---------- | ---------- |
| `resolveReadPath` | `(filePath: string, cwd: string) => string` |

### :gear: detectLineEnding

| Function | Type |
| ---------- | ---------- |
| `detectLineEnding` | `(content: string) => "\r\n" or "\n"` |

### :gear: normalizeToLF

| Function | Type |
| ---------- | ---------- |
| `normalizeToLF` | `(text: string) => string` |

### :gear: restoreLineEndings

| Function | Type |
| ---------- | ---------- |
| `restoreLineEndings` | `(text: string, ending: "\r\n" or "\n") => string` |

### :gear: normalizeForFuzzyMatch

Normalize text for fuzzy matching. Applies progressive transformations:
- Strip trailing whitespace from each line
- Normalize smart quotes to ASCII equivalents
- Normalize Unicode dashes/hyphens to ASCII hyphen
- Normalize special Unicode spaces to regular space

| Function | Type |
| ---------- | ---------- |
| `normalizeForFuzzyMatch` | `(text: string) => string` |

### :gear: fuzzyFindText

Find oldText in content, trying exact match first, then fuzzy match.
When fuzzy matching is used, the returned contentForReplacement is the
fuzzy-normalized version of the content (trailing whitespace stripped,
Unicode quotes/dashes normalized to ASCII).

| Function | Type |
| ---------- | ---------- |
| `fuzzyFindText` | `(content: string, oldText: string) => FuzzyMatchResult` |

### :gear: stripBom

Strip UTF-8 BOM if present, return both the BOM (if any) and the text without it

| Function | Type |
| ---------- | ---------- |
| `stripBom` | `(content: string) => { bom: string; text: string; }` |

### :gear: applyEditsToNormalizedContent

Apply one or more exact-text replacements to LF-normalized content.

All edits are matched against the same original content. Replacements are
then applied in reverse order so offsets remain stable. If any edit needs
fuzzy matching, the operation runs in fuzzy-normalized content space to
preserve current single-edit behavior.

| Function | Type |
| ---------- | ---------- |
| `applyEditsToNormalizedContent` | `(normalizedContent: string, edits: Edit[], path: string) => AppliedEditsResult` |

### :gear: generateDiffString

Generate a unified diff string with line numbers and context.
Returns both the diff string and the first changed line number (in the new file).

| Function | Type |
| ---------- | ---------- |
| `generateDiffString` | `(oldContent: string, newContent: string, contextLines?: number) => { diff: string; firstChangedLine: number or undefined; }` |

### :gear: computeEditsDiff

Compute the diff for one or more edit operations without applying them.
Used for preview rendering in the TUI before the tool executes.

| Function | Type |
| ---------- | ---------- |
| `computeEditsDiff` | `(path: string, edits: Edit[], cwd: string) => Promise<EditDiffResult or EditDiffError>` |

### :gear: computeEditDiff

Compute the diff for a single edit operation without applying it.
Kept as a convenience wrapper for single-edit callers.

| Function | Type |
| ---------- | ---------- |
| `computeEditDiff` | `(path: string, oldText: string, newText: string, cwd: string) => Promise<EditDiffResult or EditDiffError>` |

### :gear: withFileMutationQueue

Serialize file mutation operations targeting the same file.
Operations for different files still run in parallel.

| Function | Type |
| ---------- | ---------- |
| `withFileMutationQueue` | `<T>(filePath: string, fn: () => Promise<T>) => Promise<T>` |

### :gear: createEditToolDefinition

| Function | Type |
| ---------- | ---------- |
| `createEditToolDefinition` | `(cwd: string, options?: EditToolOptions or undefined) => ToolDefinition<TObject<{ path: TString; edits: TArray<TObject<{ oldText: TString; newText: TString; }>>; }>, EditToolDetails or undefined, EditRenderState>` |

### :gear: createEditTool

| Function | Type |
| ---------- | ---------- |
| `createEditTool` | `(cwd: string, options?: EditToolOptions or undefined) => AgentTool<TObject<{ path: TString; edits: TArray<TObject<{ oldText: TString; newText: TString; }>>; }>, any>` |

### :gear: getToolPath

| Function | Type |
| ---------- | ---------- |
| `getToolPath` | `(tool: "fd" or "rg") => string or null` |

### :gear: ensureTool

| Function | Type |
| ---------- | ---------- |
| `ensureTool` | `(tool: "fd" or "rg", silent?: boolean) => Promise<string or undefined>` |

### :gear: createFindToolDefinition

| Function | Type |
| ---------- | ---------- |
| `createFindToolDefinition` | `(cwd: string, options?: FindToolOptions or undefined) => ToolDefinition<TObject<{ pattern: TString; path: TOptional<TString>; limit: TOptional<TNumber>; }>, FindToolDetails or undefined, any>` |

### :gear: createFindTool

| Function | Type |
| ---------- | ---------- |
| `createFindTool` | `(cwd: string, options?: FindToolOptions or undefined) => AgentTool<TObject<{ pattern: TString; path: TOptional<TString>; limit: TOptional<TNumber>; }>, any>` |

### :gear: createGrepToolDefinition

| Function | Type |
| ---------- | ---------- |
| `createGrepToolDefinition` | `(cwd: string, options?: GrepToolOptions or undefined) => ToolDefinition<TObject<{ pattern: TString; path: TOptional<TString>; glob: TOptional<TString>; ignoreCase: TOptional<...>; literal: TOptional<...>; context: TOptional<...>; limit: TOptional<...>; }>, GrepToolDetails or undefined, any>` |

### :gear: createGrepTool

| Function | Type |
| ---------- | ---------- |
| `createGrepTool` | `(cwd: string, options?: GrepToolOptions or undefined) => AgentTool<TObject<{ pattern: TString; path: TOptional<TString>; glob: TOptional<TString>; ignoreCase: TOptional<...>; literal: TOptional<...>; context: TOptional<...>; limit: TOptional<...>; }>, any>` |

### :gear: createLsToolDefinition

| Function | Type |
| ---------- | ---------- |
| `createLsToolDefinition` | `(cwd: string, options?: LsToolOptions or undefined) => ToolDefinition<TObject<{ path: TOptional<TString>; limit: TOptional<TNumber>; }>, LsToolDetails or undefined, any>` |

### :gear: createLsTool

| Function | Type |
| ---------- | ---------- |
| `createLsTool` | `(cwd: string, options?: LsToolOptions or undefined) => AgentTool<TObject<{ path: TOptional<TString>; limit: TOptional<TNumber>; }>, any>` |

### :gear: loadPhoton

Load the photon module asynchronously.
Returns cached module on subsequent calls.

| Function | Type |
| ---------- | ---------- |
| `loadPhoton` | `() => Promise<typeof import("/home/arligle/EA/pi/node_modules/@silvia-odwyer/photon-node/photon_rs") or null>` |

### :gear: applyExifOrientation

| Function | Type |
| ---------- | ---------- |
| `applyExifOrientation` | `(photon: typeof import("/home/arligle/EA/pi/node_modules/@silvia-odwyer/photon-node/photon_rs"), image: PhotonImage, originalBytes: Uint8Array<ArrayBufferLike>) => PhotonImage` |

### :gear: resizeImage

Resize an image to fit within the specified max dimensions and encoded file size.
Returns null if the image cannot be resized below maxBytes.

Uses Photon (Rust/WASM) for image processing. If Photon is not available,
returns null.

Strategy for staying under maxBytes:
1. First resize to maxWidth/maxHeight
2. Try both PNG and JPEG formats, pick the smaller one
3. If still too large, try JPEG with decreasing quality
4. If still too large, progressively reduce dimensions until 1x1

| Function | Type |
| ---------- | ---------- |
| `resizeImage` | `(img: ImageContent, options?: ImageResizeOptions or undefined) => Promise<ResizedImage or null>` |

### :gear: formatDimensionNote

Format a dimension note for resized images.
This helps the model understand the coordinate mapping.

| Function | Type |
| ---------- | ---------- |
| `formatDimensionNote` | `(result: ResizedImage) => string or undefined` |

### :gear: detectSupportedImageMimeTypeFromFile

| Function | Type |
| ---------- | ---------- |
| `detectSupportedImageMimeTypeFromFile` | `(filePath: string) => Promise<string or null>` |

### :gear: createReadToolDefinition

| Function | Type |
| ---------- | ---------- |
| `createReadToolDefinition` | `(cwd: string, options?: ReadToolOptions or undefined) => ToolDefinition<TObject<{ path: TString; offset: TOptional<TNumber>; limit: TOptional<TNumber>; }>, ReadToolDetails or undefined, any>` |

### :gear: createReadTool

| Function | Type |
| ---------- | ---------- |
| `createReadTool` | `(cwd: string, options?: ReadToolOptions or undefined) => AgentTool<TObject<{ path: TString; offset: TOptional<TNumber>; limit: TOptional<TNumber>; }>, any>` |

### :gear: createWriteToolDefinition

| Function | Type |
| ---------- | ---------- |
| `createWriteToolDefinition` | `(cwd: string, options?: WriteToolOptions or undefined) => ToolDefinition<TObject<{ path: TString; content: TString; }>, undefined, any>` |

### :gear: createWriteTool

| Function | Type |
| ---------- | ---------- |
| `createWriteTool` | `(cwd: string, options?: WriteToolOptions or undefined) => AgentTool<TObject<{ path: TString; content: TString; }>, any>` |

### :gear: createToolDefinition

按工具名称创建对应的静态定义。

| Function | Type |
| ---------- | ---------- |
| `createToolDefinition` | `(toolName: ToolName, cwd: string, options?: ToolsOptions or undefined) => ToolDef` |

Parameters:

* `toolName`: - 目标工具名称
* `cwd`: - 工作目录，用于解析相对路径
* `options`: - 各工具的可选配置


Returns:

该工具的定义对象

### :gear: createTool

按工具名称创建对应的运行时实例。

| Function | Type |
| ---------- | ---------- |
| `createTool` | `(toolName: ToolName, cwd: string, options?: ToolsOptions or undefined) => Tool` |

Parameters:

* `toolName`: - 目标工具名称
* `cwd`: - 工作目录
* `options`: - 各工具的可选配置


Returns:

该工具的运行时实例

### :gear: createCodingToolDefinitions

创建编码工具集的静态定义（read/bash/edit/write）。
这组工具具有读写文件和执行命令的能力。

| Function | Type |
| ---------- | ---------- |
| `createCodingToolDefinitions` | `(cwd: string, options?: ToolsOptions or undefined) => ToolDef[]` |

### :gear: createReadOnlyToolDefinitions

创建只读工具集的静态定义（read/grep/find/ls）。
这组工具仅查询文件系统，不会产生任何修改。

| Function | Type |
| ---------- | ---------- |
| `createReadOnlyToolDefinitions` | `(cwd: string, options?: ToolsOptions or undefined) => ToolDef[]` |

### :gear: createAllToolDefinitions

创建全部内置工具的静态定义，以 `Record<ToolName, ToolDef>` 形式返回，
方便按名称索引。

| Function | Type |
| ---------- | ---------- |
| `createAllToolDefinitions` | `(cwd: string, options?: ToolsOptions or undefined) => Record<ToolName, ToolDef>` |

### :gear: createCodingTools

创建编码工具集的运行时实例（read/bash/edit/write）。

| Function | Type |
| ---------- | ---------- |
| `createCodingTools` | `(cwd: string, options?: ToolsOptions or undefined) => Tool[]` |

References:

* `createCodingToolDefinitions`


### :gear: createReadOnlyTools

创建只读工具集的运行时实例（read/grep/find/ls）。

| Function | Type |
| ---------- | ---------- |
| `createReadOnlyTools` | `(cwd: string, options?: ToolsOptions or undefined) => Tool[]` |

References:

* `createReadOnlyToolDefinitions`


### :gear: createAllTools

创建全部内置工具的运行时实例，以 `Record<ToolName, Tool>` 形式返回，
方便按名称索引。

| Function | Type |
| ---------- | ---------- |
| `createAllTools` | `(cwd: string, options?: ToolsOptions or undefined) => Record<ToolName, Tool>` |

### :gear: defineTool

Preserve parameter inference for standalone tool definitions.

Use this when assigning a tool to a variable or passing it through arrays such
as `customTools`, where contextual typing would otherwise widen params to
`unknown`.

| Function | Type |
| ---------- | ---------- |
| `defineTool` | `<TParams extends TSchema, TDetails = unknown, TState = any>(tool: ToolDefinition<TParams, TDetails, TState>) => ToolDefinition<TParams, TDetails, TState> and AnyToolDefinition` |

### :gear: isBashToolResult

| Function | Type |
| ---------- | ---------- |
| `isBashToolResult` | `(e: ToolResultEvent) => e is BashToolResultEvent` |

### :gear: isReadToolResult

| Function | Type |
| ---------- | ---------- |
| `isReadToolResult` | `(e: ToolResultEvent) => e is ReadToolResultEvent` |

### :gear: isEditToolResult

| Function | Type |
| ---------- | ---------- |
| `isEditToolResult` | `(e: ToolResultEvent) => e is EditToolResultEvent` |

### :gear: isWriteToolResult

| Function | Type |
| ---------- | ---------- |
| `isWriteToolResult` | `(e: ToolResultEvent) => e is WriteToolResultEvent` |

### :gear: isGrepToolResult

| Function | Type |
| ---------- | ---------- |
| `isGrepToolResult` | `(e: ToolResultEvent) => e is GrepToolResultEvent` |

### :gear: isFindToolResult

| Function | Type |
| ---------- | ---------- |
| `isFindToolResult` | `(e: ToolResultEvent) => e is FindToolResultEvent` |

### :gear: isLsToolResult

| Function | Type |
| ---------- | ---------- |
| `isLsToolResult` | `(e: ToolResultEvent) => e is LsToolResultEvent` |

### :gear: isToolCallEventType

Type guard for narrowing ToolCallEvent by tool name.

Built-in tools narrow automatically (no type params needed):
```ts
if (isToolCallEventType("bash", event)) {
  event.input.command;  // string
}
```

Custom tools require explicit type parameters:
```ts
if (isToolCallEventType<"my_tool", MyToolInput>("my_tool", event)) {
  event.input.action;  // typed
}
```

Note: Direct narrowing via `event.toolName === "bash"` doesn't work because
CustomToolCallEvent.toolName is `string` which overlaps with all literals.

| Function | Type |
| ---------- | ---------- |
| `isToolCallEventType` | `{ (toolName: "bash", event: ToolCallEvent): event is BashToolCallEvent; (toolName: "read", event: ToolCallEvent): event is ReadToolCallEvent; (toolName: "edit", event: ToolCallEvent): event is EditToolCallEvent; (toolName: "write", event: ToolCallEvent): event is WriteToolCallEvent; (toolName: "grep", event: ToolCal...` |

### :gear: isToolCallEventType

Type guard for narrowing ToolCallEvent by tool name.

Built-in tools narrow automatically (no type params needed):
```ts
if (isToolCallEventType("bash", event)) {
  event.input.command;  // string
}
```

Custom tools require explicit type parameters:
```ts
if (isToolCallEventType<"my_tool", MyToolInput>("my_tool", event)) {
  event.input.action;  // typed
}
```

Note: Direct narrowing via `event.toolName === "bash"` doesn't work because
CustomToolCallEvent.toolName is `string` which overlaps with all literals.

| Function | Type |
| ---------- | ---------- |
| `isToolCallEventType` | `{ (toolName: "bash", event: ToolCallEvent): event is BashToolCallEvent; (toolName: "read", event: ToolCallEvent): event is ReadToolCallEvent; (toolName: "edit", event: ToolCallEvent): event is EditToolCallEvent; (toolName: "write", event: ToolCallEvent): event is WriteToolCallEvent; (toolName: "grep", event: ToolCal...` |

### :gear: isToolCallEventType

Type guard for narrowing ToolCallEvent by tool name.

Built-in tools narrow automatically (no type params needed):
```ts
if (isToolCallEventType("bash", event)) {
  event.input.command;  // string
}
```

Custom tools require explicit type parameters:
```ts
if (isToolCallEventType<"my_tool", MyToolInput>("my_tool", event)) {
  event.input.action;  // typed
}
```

Note: Direct narrowing via `event.toolName === "bash"` doesn't work because
CustomToolCallEvent.toolName is `string` which overlaps with all literals.

| Function | Type |
| ---------- | ---------- |
| `isToolCallEventType` | `{ (toolName: "bash", event: ToolCallEvent): event is BashToolCallEvent; (toolName: "read", event: ToolCallEvent): event is ReadToolCallEvent; (toolName: "edit", event: ToolCallEvent): event is EditToolCallEvent; (toolName: "write", event: ToolCallEvent): event is WriteToolCallEvent; (toolName: "grep", event: ToolCal...` |

### :gear: isToolCallEventType

Type guard for narrowing ToolCallEvent by tool name.

Built-in tools narrow automatically (no type params needed):
```ts
if (isToolCallEventType("bash", event)) {
  event.input.command;  // string
}
```

Custom tools require explicit type parameters:
```ts
if (isToolCallEventType<"my_tool", MyToolInput>("my_tool", event)) {
  event.input.action;  // typed
}
```

Note: Direct narrowing via `event.toolName === "bash"` doesn't work because
CustomToolCallEvent.toolName is `string` which overlaps with all literals.

| Function | Type |
| ---------- | ---------- |
| `isToolCallEventType` | `{ (toolName: "bash", event: ToolCallEvent): event is BashToolCallEvent; (toolName: "read", event: ToolCallEvent): event is ReadToolCallEvent; (toolName: "edit", event: ToolCallEvent): event is EditToolCallEvent; (toolName: "write", event: ToolCallEvent): event is WriteToolCallEvent; (toolName: "grep", event: ToolCal...` |

### :gear: isToolCallEventType

Type guard for narrowing ToolCallEvent by tool name.

Built-in tools narrow automatically (no type params needed):
```ts
if (isToolCallEventType("bash", event)) {
  event.input.command;  // string
}
```

Custom tools require explicit type parameters:
```ts
if (isToolCallEventType<"my_tool", MyToolInput>("my_tool", event)) {
  event.input.action;  // typed
}
```

Note: Direct narrowing via `event.toolName === "bash"` doesn't work because
CustomToolCallEvent.toolName is `string` which overlaps with all literals.

| Function | Type |
| ---------- | ---------- |
| `isToolCallEventType` | `{ (toolName: "bash", event: ToolCallEvent): event is BashToolCallEvent; (toolName: "read", event: ToolCallEvent): event is ReadToolCallEvent; (toolName: "edit", event: ToolCallEvent): event is EditToolCallEvent; (toolName: "write", event: ToolCallEvent): event is WriteToolCallEvent; (toolName: "grep", event: ToolCal...` |

### :gear: isToolCallEventType

Type guard for narrowing ToolCallEvent by tool name.

Built-in tools narrow automatically (no type params needed):
```ts
if (isToolCallEventType("bash", event)) {
  event.input.command;  // string
}
```

Custom tools require explicit type parameters:
```ts
if (isToolCallEventType<"my_tool", MyToolInput>("my_tool", event)) {
  event.input.action;  // typed
}
```

Note: Direct narrowing via `event.toolName === "bash"` doesn't work because
CustomToolCallEvent.toolName is `string` which overlaps with all literals.

| Function | Type |
| ---------- | ---------- |
| `isToolCallEventType` | `{ (toolName: "bash", event: ToolCallEvent): event is BashToolCallEvent; (toolName: "read", event: ToolCallEvent): event is ReadToolCallEvent; (toolName: "edit", event: ToolCallEvent): event is EditToolCallEvent; (toolName: "write", event: ToolCallEvent): event is WriteToolCallEvent; (toolName: "grep", event: ToolCal...` |

### :gear: isToolCallEventType

Type guard for narrowing ToolCallEvent by tool name.

Built-in tools narrow automatically (no type params needed):
```ts
if (isToolCallEventType("bash", event)) {
  event.input.command;  // string
}
```

Custom tools require explicit type parameters:
```ts
if (isToolCallEventType<"my_tool", MyToolInput>("my_tool", event)) {
  event.input.action;  // typed
}
```

Note: Direct narrowing via `event.toolName === "bash"` doesn't work because
CustomToolCallEvent.toolName is `string` which overlaps with all literals.

| Function | Type |
| ---------- | ---------- |
| `isToolCallEventType` | `{ (toolName: "bash", event: ToolCallEvent): event is BashToolCallEvent; (toolName: "read", event: ToolCallEvent): event is ReadToolCallEvent; (toolName: "edit", event: ToolCallEvent): event is EditToolCallEvent; (toolName: "write", event: ToolCallEvent): event is WriteToolCallEvent; (toolName: "grep", event: ToolCal...` |

### :gear: isToolCallEventType

Type guard for narrowing ToolCallEvent by tool name.

Built-in tools narrow automatically (no type params needed):
```ts
if (isToolCallEventType("bash", event)) {
  event.input.command;  // string
}
```

Custom tools require explicit type parameters:
```ts
if (isToolCallEventType<"my_tool", MyToolInput>("my_tool", event)) {
  event.input.action;  // typed
}
```

Note: Direct narrowing via `event.toolName === "bash"` doesn't work because
CustomToolCallEvent.toolName is `string` which overlaps with all literals.

| Function | Type |
| ---------- | ---------- |
| `isToolCallEventType` | `{ (toolName: "bash", event: ToolCallEvent): event is BashToolCallEvent; (toolName: "read", event: ToolCallEvent): event is ReadToolCallEvent; (toolName: "edit", event: ToolCallEvent): event is EditToolCallEvent; (toolName: "write", event: ToolCallEvent): event is WriteToolCallEvent; (toolName: "grep", event: ToolCal...` |

### :gear: isToolCallEventType

Type guard for narrowing ToolCallEvent by tool name.

Built-in tools narrow automatically (no type params needed):
```ts
if (isToolCallEventType("bash", event)) {
  event.input.command;  // string
}
```

Custom tools require explicit type parameters:
```ts
if (isToolCallEventType<"my_tool", MyToolInput>("my_tool", event)) {
  event.input.action;  // typed
}
```

Note: Direct narrowing via `event.toolName === "bash"` doesn't work because
CustomToolCallEvent.toolName is `string` which overlaps with all literals.

| Function | Type |
| ---------- | ---------- |
| `isToolCallEventType` | `{ (toolName: "bash", event: ToolCallEvent): event is BashToolCallEvent; (toolName: "read", event: ToolCallEvent): event is ReadToolCallEvent; (toolName: "edit", event: ToolCallEvent): event is EditToolCallEvent; (toolName: "write", event: ToolCallEvent): event is WriteToolCallEvent; (toolName: "grep", event: ToolCal...` |

### :gear: isValidThinkingLevel

| Function | Type |
| ---------- | ---------- |
| `isValidThinkingLevel` | `(level: string) => level is ThinkingLevel` |

### :gear: parseArgs

| Function | Type |
| ---------- | ---------- |
| `parseArgs` | `(args: string[]) => Args` |

### :gear: printHelp

| Function | Type |
| ---------- | ---------- |
| `printHelp` | `(extensionFlags?: ExtensionFlag[] or undefined) => void` |

### :gear: processFileArguments

Process

| Function | Type |
| ---------- | ---------- |
| `processFileArguments` | `(fileArgs: string[], options?: ProcessFileOptions or undefined) => Promise<ProcessedFiles>` |

### :gear: buildInitialMessage

Combine stdin content,

| Function | Type |
| ---------- | ---------- |
| `buildInitialMessage` | `({ parsed, fileText, fileImages, stdinContent, }: InitialMessageInput) => InitialMessageResult` |

### :gear: getProviderLoginHelp

| Function | Type |
| ---------- | ---------- |
| `getProviderLoginHelp` | `() => string` |

### :gear: formatNoModelsAvailableMessage

| Function | Type |
| ---------- | ---------- |
| `formatNoModelsAvailableMessage` | `() => string` |

### :gear: formatNoModelSelectedMessage

| Function | Type |
| ---------- | ---------- |
| `formatNoModelSelectedMessage` | `() => string` |

### :gear: formatNoApiKeyFoundMessage

| Function | Type |
| ---------- | ---------- |
| `formatNoApiKeyFoundMessage` | `(provider: string) => string` |

### :gear: listModels

List available models, optionally filtered by search pattern

| Function | Type |
| ---------- | ---------- |
| `listModels` | `(modelRegistry: ModelRegistry, searchPattern?: string or undefined) => Promise<void>` |

### :gear: hasSessionName

| Function | Type |
| ---------- | ---------- |
| `hasSessionName` | `(session: SessionInfo) => boolean` |

### :gear: parseSearchQuery

| Function | Type |
| ---------- | ---------- |
| `parseSearchQuery` | `(query: string) => ParsedSearchQuery` |

### :gear: matchSession

| Function | Type |
| ---------- | ---------- |
| `matchSession` | `(session: SessionInfo, parsed: ParsedSearchQuery) => MatchResult` |

### :gear: filterAndSortSessions

| Function | Type |
| ---------- | ---------- |
| `filterAndSortSessions` | `(sessions: SessionInfo[], query: string, sortMode: SortMode, nameFilter?: NameFilter) => SessionInfo[]` |

### :gear: selectSession

Show TUI session selector and return selected session path or null if cancelled

| Function | Type |
| ---------- | ---------- |
| `selectSession` | `(currentSessionsLoader: SessionsLoader, allSessionsLoader: SessionsLoader) => Promise<string or null>` |

### :gear: sleep

Sleep helper that respects abort signal.

| Function | Type |
| ---------- | ---------- |
| `sleep` | `(ms: number, signal?: AbortSignal or undefined) => Promise<void>` |

### :gear: exportSessionToHtml

Export session to HTML using SessionManager and AgentState.
Used by TUI's /export command.

| Function | Type |
| ---------- | ---------- |
| `exportSessionToHtml` | `(sm: SessionManager, state?: AgentState or undefined, options?: string or ExportOptions or undefined) => Promise<string>` |

### :gear: exportFromFile

Export session file to HTML (standalone, without AgentState).
Used by CLI for exporting arbitrary session files.

| Function | Type |
| ---------- | ---------- |
| `exportFromFile` | `(inputPath: string, options?: string or ExportOptions or undefined) => Promise<string>` |

### :gear: ansiToHtml

Convert ANSI-escaped text to HTML with inline styles.

| Function | Type |
| ---------- | ---------- |
| `ansiToHtml` | `(text: string) => string` |

### :gear: ansiLinesToHtml

Convert array of ANSI-escaped lines to HTML.
Each line is wrapped in a div element.

| Function | Type |
| ---------- | ---------- |
| `ansiLinesToHtml` | `(lines: string[]) => string` |

### :gear: createToolHtmlRenderer

| Function | Type |
| ---------- | ---------- |
| `createToolHtmlRenderer` | `(deps: ToolHtmlRendererDeps) => ToolHtmlRenderer` |

### :gear: parseCommandArgs

Parse command arguments respecting quoted strings (bash-style)
Returns array of arguments

| Function | Type |
| ---------- | ---------- |
| `parseCommandArgs` | `(argsString: string) => string[]` |

### :gear: substituteArgs

Substitute argument placeholders in template content
Supports:
- $1, $2, ... for positional args
- $@ and $ARGUMENTS for all args
- ${@:N} for args from Nth onwards (bash-style slicing)
- ${@:N:L} for L args starting from Nth

Note: Replacement happens on the template string only. Argument values
containing patterns like $1, $@, or $ARGUMENTS are NOT recursively substituted.

| Function | Type |
| ---------- | ---------- |
| `substituteArgs` | `(content: string, args: string[]) => string` |

### :gear: loadPromptTemplates

Load all prompt templates from:
1. Global: agentDir/prompts/
2. Project: cwd/{CONFIG_DIR_NAME}/prompts/
3. Explicit prompt paths

| Function | Type |
| ---------- | ---------- |
| `loadPromptTemplates` | `(options: LoadPromptTemplatesOptions) => PromptTemplate[]` |

### :gear: expandPromptTemplate

Expand a prompt template if it matches a template name.
Returns the expanded content or the original text if not a template.

| Function | Type |
| ---------- | ---------- |
| `expandPromptTemplate` | `(text: string, templates: PromptTemplate[]) => string` |

### :gear: loadProjectContextFiles

| Function | Type |
| ---------- | ---------- |
| `loadProjectContextFiles` | `(options: { cwd: string; agentDir: string; }) => { path: string; content: string; }[]` |

### :gear: findExactModelReferenceMatch

在可用模型列表中查找精确匹配的模型引用

支持以下格式：
- 完整引用 "provider/modelId"
- 裸 modelId（跨提供商唯一时才命中，模糊匹配会被拒绝）

| Function | Type |
| ---------- | ---------- |
| `findExactModelReferenceMatch` | `(modelReference: string, availableModels: Model<Api>[]) => Model<Api> or undefined` |

Parameters:

* `modelReference`: - 用户提供的模型引用字符串
* `availableModels`: - 可用模型列表


Returns:

匹配的模型，未找到或匹配不唯一时返回 undefined

### :gear: parseModelPattern

解析模型模式，提取模型和思维级别

处理策略（支持 ID 中包含冒号的模型，如 OpenRouter 的 ":exacto" 后缀）：
1. 先尝试将完整模式作为模型 ID 匹配
2. 如果匹配成功，返回该模型（思维级别为 undefined）
3. 如果匹配失败且包含冒号，从最后一个冒号处拆分：
   - 后缀是有效的思维级别 → 使用该级别，递归解析前缀
   - 后缀无效 → 根据模式发出警告，递归解析前缀

| Function | Type |
| ---------- | ---------- |
| `parseModelPattern` | `(pattern: string, availableModels: Model<Api>[], options?: { allowInvalidThinkingLevelFallback?: boolean or undefined; } or undefined) => ParsedModelResult` |

Parameters:

* `pattern`: - 模型匹配模式（如 "claude-sonnet-4:high"）
* `availableModels`: - 可用模型列表
* `options.allowInvalidThinkingLevelFallback`: - 无效思维级别时是否回退（CLI 模式为 false）


Returns:

解析结果，包含模型、思维级别和可能的警告

### :gear: resolveModelScope

批量解析模型模式为实际 Model 对象，支持可选的思维级别

模式格式："pattern:level"（:level 可选）
对于每个模式，查找所有匹配的模型并选择最佳版本：
1. 优先选择别名（如 claude-sonnet-4-5）而非日期版本（claude-sonnet-4-5-20250929）
2. 无别名时选择最新日期版本

支持模式中包含冒号的模型 ID（如 OpenRouter 的 "model:exacto"）。
算法先尝试匹配完整模式，然后逐步剥离冒号后缀进行匹配。

| Function | Type |
| ---------- | ---------- |
| `resolveModelScope` | `(patterns: string[], modelRegistry: ModelRegistry) => Promise<ScopedModel[]>` |

Parameters:

* `patterns`: - 模型匹配模式数组，支持 glob 通配符
* `modelRegistry`: - 模型注册表


Returns:

去重后的作用域模型列表

### :gear: resolveCliModel

从 CLI 参数解析单个模型

支持的输入格式：
- `--provider <provider> --model <pattern>`
- `--model <provider>/<pattern>`
- 模糊匹配（与模型作用域相同的规则：精确 ID → 部分 ID/名称匹配）

注意：此函数不直接应用思维级别，但会从 "<pattern>:<thinking>" 格式中
解析并返回思维级别，供调用方应用。

| Function | Type |
| ---------- | ---------- |
| `resolveCliModel` | `(options: { cliProvider?: string or undefined; cliModel?: string or undefined; modelRegistry: ModelRegistry; }) => ResolveCliModelResult` |

Parameters:

* `options.cliProvider`: - CLI --provider 参数
* `options.cliModel`: - CLI --model 参数
* `options.modelRegistry`: - 模型注册表


Returns:

解析结果，包含模型、思维级别、警告或错误

### :gear: findInitialModel

按优先级链选择初始模型

优先级顺序：
1. CLI 参数指定的提供商和模型
2. 作用域模型中的第一个（继续/恢复会话时跳过）
3. 从会话中恢复的模型（由调用方在恢复场景中处理）
4. 用户设置中保存的默认模型
5. 第一个具有有效 API 密钥的可用模型

| Function | Type |
| ---------- | ---------- |
| `findInitialModel` | `(options: { cliProvider?: string or undefined; cliModel?: string or undefined; scopedModels: ScopedModel[]; isContinuing: boolean; defaultProvider?: string or undefined; defaultModelId?: string or undefined; defaultThinkingLevel?: ThinkingLevel or undefined; modelRegistry: ModelRegistry; }) => Promise<...>` |

Parameters:

* `options`: - 初始模型选择选项


Returns:

初始模型选择结果

### :gear: restoreModelFromSession

从会话中恢复模型，失败时回退到可用模型

| Function | Type |
| ---------- | ---------- |
| `restoreModelFromSession` | `(savedProvider: string, savedModelId: string, currentModel: Model<Api> or undefined, shouldPrintMessages: boolean, modelRegistry: ModelRegistry) => Promise<...>` |

Parameters:

* `savedProvider`: - 会话中保存的提供商
* `savedModelId`: - 会话中保存的模型 ID
* `currentModel`: - 当前已加载的模型（作为首选回退）
* `shouldPrintMessages`: - 是否打印状态消息到控制台
* `modelRegistry`: - 模型注册表


Returns:

恢复或回退的模型，以及可能的回退消息

### :gear: isInstallTelemetryEnabled

| Function | Type |
| ---------- | ---------- |
| `isInstallTelemetryEnabled` | `(settingsManager: SettingsManager, telemetryEnv?: string or undefined) => boolean` |

### :gear: resetTimings

| Function | Type |
| ---------- | ---------- |
| `resetTimings` | `() => void` |

### :gear: time

| Function | Type |
| ---------- | ---------- |
| `time` | `(label: string) => void` |

### :gear: printTimings

| Function | Type |
| ---------- | ---------- |
| `printTimings` | `() => void` |

### :gear: createAgentSession

创建一个完整的代理会话（AgentSession）。

本函数是 pi SDK 的核心入口，负责将认证、模型、工具、扩展、
会话持久化等所有子系统组装在一起。支持从零创建新会话，
也支持恢复已有会话（包括模型和思考等级的恢复）。

| Function | Type |
| ---------- | ---------- |
| `createAgentSession` | `(options?: CreateAgentSessionOptions) => Promise<CreateAgentSessionResult>` |

Parameters:

* `options`: - 创建会话的配置选项，全部可选


Returns:

包含会话实例、扩展加载结果和可选模型回退警告的对象

Examples:

```typescript
// 最简调用 —— 使用全部默认值
const { session } = await createAgentSession();

// 指定模型
import { getModel } from '@earendil-works/pi-ai';
const { session } = await createAgentSession({
  model: getModel('anthropic', 'claude-opus-4-5'),
  thinkingLevel: 'high',
});

// 继续上一次会话
const { session, modelFallbackMessage } = await createAgentSession({
  continueSession: true,
});

// 完全控制 —— 自定义资源加载器和会话管理器
const loader = new DefaultResourceLoader({
  cwd: process.cwd(),
  agentDir: getAgentDir(),
  settingsManager: SettingsManager.create(),
});
await loader.reload();
const { session } = await createAgentSession({
  model: myModel,
  tools: [readTool, bashTool],
  resourceLoader: loader,
  sessionManager: SessionManager.inMemory(),
});
```


### :gear: getMissingSessionCwdIssue

| Function | Type |
| ---------- | ---------- |
| `getMissingSessionCwdIssue` | `(sessionManager: SessionCwdSource, fallbackCwd: string) => SessionCwdIssue or undefined` |

### :gear: formatMissingSessionCwdError

| Function | Type |
| ---------- | ---------- |
| `formatMissingSessionCwdError` | `(issue: SessionCwdIssue) => string` |

### :gear: formatMissingSessionCwdPrompt

| Function | Type |
| ---------- | ---------- |
| `formatMissingSessionCwdPrompt` | `(issue: SessionCwdIssue) => string` |

### :gear: assertSessionCwdExists

| Function | Type |
| ---------- | ---------- |
| `assertSessionCwdExists` | `(sessionManager: SessionCwdSource, fallbackCwd: string) => void` |

### :gear: parseChangelog

Parse changelog entries from CHANGELOG.md
Scans for ## lines and collects content until next ## or EOF

| Function | Type |
| ---------- | ---------- |
| `parseChangelog` | `(changelogPath: string) => ChangelogEntry[]` |

### :gear: compareVersions

Compare versions. Returns: -1 if v1 < v2, 0 if v1 === v2, 1 if v1 > v2

| Function | Type |
| ---------- | ---------- |
| `compareVersions` | `(v1: ChangelogEntry, v2: ChangelogEntry) => number` |

### :gear: getNewEntries

Get entries newer than lastVersion

| Function | Type |
| ---------- | ---------- |
| `getNewEntries` | `(entries: ChangelogEntry[], lastVersion: string) => ChangelogEntry[]` |

### :gear: isWaylandSession

| Function | Type |
| ---------- | ---------- |
| `isWaylandSession` | `(env?: ProcessEnv) => boolean` |

### :gear: extensionForImageMimeType

| Function | Type |
| ---------- | ---------- |
| `extensionForImageMimeType` | `(mimeType: string) => string or null` |

### :gear: readClipboardImage

| Function | Type |
| ---------- | ---------- |
| `readClipboardImage` | `(options?: { env?: ProcessEnv or undefined; platform?: Platform or undefined; } or undefined) => Promise<ClipboardImage or null>` |

### :gear: copyToClipboard

| Function | Type |
| ---------- | ---------- |
| `copyToClipboard` | `(text: string) => Promise<void>` |

### :gear: getPiUserAgent

| Function | Type |
| ---------- | ---------- |
| `getPiUserAgent` | `(version: string) => string` |

### :gear: comparePackageVersions

| Function | Type |
| ---------- | ---------- |
| `comparePackageVersions` | `(leftVersion: string, rightVersion: string) => number or undefined` |

### :gear: isNewerPackageVersion

| Function | Type |
| ---------- | ---------- |
| `isNewerPackageVersion` | `(candidateVersion: string, currentVersion: string) => boolean` |

### :gear: getLatestPiRelease

| Function | Type |
| ---------- | ---------- |
| `getLatestPiRelease` | `(currentVersion: string, options?: { timeoutMs?: number or undefined; }) => Promise<LatestPiRelease or undefined>` |

### :gear: getLatestPiVersion

| Function | Type |
| ---------- | ---------- |
| `getLatestPiVersion` | `(currentVersion: string, options?: { timeoutMs?: number or undefined; }) => Promise<string or undefined>` |

### :gear: checkForNewPiVersion

| Function | Type |
| ---------- | ---------- |
| `checkForNewPiVersion` | `(currentVersion: string) => Promise<string or undefined>` |

### :gear: convertToPng

Convert image to PNG format for terminal display.
Kitty graphics protocol requires PNG format (f=100).

| Function | Type |
| ---------- | ---------- |
| `convertToPng` | `(base64Data: string, mimeType: string) => Promise<{ data: string; mimeType: string; } or null>` |

### :gear: isApiKeyLoginProvider

判断给定提供商是否支持通过 API Key 方式登录。
优先级：内置显示名 > 内置提供商集合 > OAuth 提供商集合取反。

| Function | Type |
| ---------- | ---------- |
| `isApiKeyLoginProvider` | `(providerId: string, oauthProviderIds: ReadonlySet<string>, builtInProviderIds?: ReadonlySet<string>) => boolean` |

Parameters:

* `providerId`: - 提供商标识符
* `oauthProviderIds`: - 已注册的 OAuth 提供商集合
* `builtInProviderIds`: - 内置提供商集合（用于排除不支持的第三方）


### :gear: runPrintMode

Run in print (single-shot) mode.
Sends prompts to the agent and outputs the result.

| Function | Type |
| ---------- | ---------- |
| `runPrintMode` | `(runtimeHost: AgentSessionRuntime, options: PrintModeOptions) => Promise<number>` |

### :gear: serializeJsonLine

Serialize a single strict JSONL record.

Framing is LF-only. Payload strings may contain other Unicode separators such as
U+2028 and U+2029. Clients must split records on `\n` only.

| Function | Type |
| ---------- | ---------- |
| `serializeJsonLine` | `(value: unknown) => string` |

### :gear: attachJsonlLineReader

Attach an LF-only JSONL reader to a stream.

This intentionally does not use Node readline. Readline splits on additional
Unicode separators that are valid inside JSON strings and therefore does not
implement strict JSONL framing.

| Function | Type |
| ---------- | ---------- |
| `attachJsonlLineReader` | `(stream: Readable, onLine: (line: string) => void) => () => void` |

### :gear: runRpcMode

Run in RPC mode.
Listens for JSON commands on stdin, outputs events and responses on stdout.

| Function | Type |
| ---------- | ---------- |
| `runRpcMode` | `(runtimeHost: AgentSessionRuntime) => Promise<never>` |

### :gear: createExtensionRuntime

创建扩展运行时（初始为抛出异常的桩方法）。

Runner.bindCore() 会替换为真正的实现。加载期间扩展只能使用
注册类方法（on、registerTool 等），不能调用发送消息等操作方法。

| Function | Type |
| ---------- | ---------- |
| `createExtensionRuntime` | `() => ExtensionRuntime` |

### :gear: loadExtensionFromFactory

从内联工厂函数创建 Extension（跳过文件加载）

| Function | Type |
| ---------- | ---------- |
| `loadExtensionFromFactory` | `(factory: ExtensionFactory, cwd: string, eventBus: EventBus, runtime: ExtensionRuntime, extensionPath?: string) => Promise<Extension>` |

### :gear: loadExtensions

从路径列表批量加载扩展。
返回加载结果包含扩展列表、错误列表和共享运行时。

| Function | Type |
| ---------- | ---------- |
| `loadExtensions` | `(paths: string[], cwd: string, eventBus?: EventBus or undefined) => Promise<LoadExtensionsResult>` |

### :gear: discoverAndLoadExtensions

从标准位置发现并加载扩展。
搜索顺序：本地 .pi/extensions → 全局 agentDir/extensions → 配置路径

| Function | Type |
| ---------- | ---------- |
| `discoverAndLoadExtensions` | `(configuredPaths: string[], cwd: string, agentDir?: string, eventBus?: EventBus or undefined) => Promise<LoadExtensionsResult>` |

### :gear: emitSessionShutdownEvent

Helper function to emit session_shutdown event to extensions.
Returns true if the event was emitted, false if there were no handlers.

| Function | Type |
| ---------- | ---------- |
| `emitSessionShutdownEvent` | `(extensionRunner: ExtensionRunner, event: SessionShutdownEvent) => Promise<boolean>` |

### :gear: wrapRegisteredTool

Wrap a RegisteredTool into an AgentTool.
Uses the runner's createContext() for consistent context across tools and event handlers.

| Function | Type |
| ---------- | ---------- |
| `wrapRegisteredTool` | `(registeredTool: RegisteredTool, runner: ExtensionRunner) => AgentTool<TSchema, any>` |

### :gear: wrapRegisteredTools

Wrap all registered tools into AgentTools.
Uses the runner's createContext() for consistent context across tools and event handlers.

| Function | Type |
| ---------- | ---------- |
| `wrapRegisteredTools` | `(registeredTools: RegisteredTool[], runner: ExtensionRunner) => AgentTool<TSchema, any>[]` |

### :gear: parseSkillBlock

从消息文本中解析技能块
如果文本不包含技能块格式则返回 null

| Function | Type |
| ---------- | ---------- |
| `parseSkillBlock` | `(text: string) => ParsedSkillBlock or null` |

Parameters:

* `text`: - 待解析的消息文本


Returns:

解析后的技能块，或 null

### :gear: createAgentSessionServices

Create cwd-bound runtime services.

Returns services plus diagnostics. It does not create an AgentSession.

| Function | Type |
| ---------- | ---------- |
| `createAgentSessionServices` | `(options: CreateAgentSessionServicesOptions) => Promise<AgentSessionServices>` |

### :gear: createAgentSessionFromServices

Create an AgentSession from previously created services.

This keeps session creation separate from service creation so callers can
resolve model, thinking, tools, and other session inputs against the target
cwd before constructing the session.

| Function | Type |
| ---------- | ---------- |
| `createAgentSessionFromServices` | `(options: CreateAgentSessionFromServicesOptions) => Promise<CreateAgentSessionResult>` |

### :gear: createAgentSessionRuntime

Create the initial runtime from a runtime factory and initial session target.

The same factory is stored on the returned AgentSessionRuntime and reused for
later /new, /resume, /fork, and import flows.

| Function | Type |
| ---------- | ---------- |
| `createAgentSessionRuntime` | `(createRuntime: CreateAgentSessionRuntimeFactory, options: { cwd: string; agentDir: string; sessionManager: SessionManager; sessionStartEvent?: SessionStartEvent or undefined; }) => Promise<...>` |

### :gear: migrateAuthToAuthJson

Migrate legacy oauth.json and settings.json apiKeys to auth.json.

| Function | Type |
| ---------- | ---------- |
| `migrateAuthToAuthJson` | `() => string[]` |

Returns:

Array of provider names that were migrated

### :gear: migrateSessionsFromAgentRoot

Migrate sessions from ~/.pi/agent/*.jsonl to proper session directories.

Bug in v0.30.0: Sessions were saved to ~/.pi/agent/ instead of
~/.pi/agent/sessions/<encoded-cwd>/. This migration moves them
to the correct location based on the cwd in their session header.

See: https://github.com/earendil-works/pi-mono/issues/320

| Function | Type |
| ---------- | ---------- |
| `migrateSessionsFromAgentRoot` | `() => void` |

### :gear: showDeprecationWarnings

Print deprecation warnings and wait for keypress.

| Function | Type |
| ---------- | ---------- |
| `showDeprecationWarnings` | `(warnings: string[]) => Promise<void>` |

### :gear: runMigrations

Run all migrations. Called once on startup.

| Function | Type |
| ---------- | ---------- |
| `runMigrations` | `(cwd: string) => { migratedAuthProviders: string[]; deprecationWarnings: string[]; }` |

Returns:

Object with migration results and deprecation warnings

### :gear: selectConfig

Show TUI config selector and return when closed

| Function | Type |
| ---------- | ---------- |
| `selectConfig` | `(options: ConfigSelectorOptions) => Promise<void>` |

### :gear: handleConfigCommand

| Function | Type |
| ---------- | ---------- |
| `handleConfigCommand` | `(args: string[]) => Promise<boolean>` |

### :gear: handlePackageCommand

| Function | Type |
| ---------- | ---------- |
| `handlePackageCommand` | `(args: string[]) => Promise<boolean>` |

### :gear: main

CLI 主入口函数。

完整的启动流程：
1. 处理包管理器命令（如安装扩展）和配置命令
2. 解析 CLI 参数并确定运行模式
3. 运行数据迁移和废弃警告
4. 创建或恢复会话管理器
5. 创建 Agent 会话运行时（延迟初始化，确保 cwd 正确）
6. 分发到对应的运行模式（交互式 / 打印 / RPC）

| Function | Type |
| ---------- | ---------- |
| `main` | `(args: string[], options?: MainOptions or undefined) => Promise<void>` |

Parameters:

* `args`: - 命令行参数数组（不含 node 和脚本路径）
* `options`: - 可选配置，如自定义扩展工厂



## :wrench: Constants

- [isBunBinary](#gear-isbunbinary)
- [isBunRuntime](#gear-isbunruntime)
- [PACKAGE_NAME](#gear-package_name)
- [APP_NAME](#gear-app_name)
- [APP_TITLE](#gear-app_title)
- [CONFIG_DIR_NAME](#gear-config_dir_name)
- [VERSION](#gear-version)
- [ENV_AGENT_DIR](#gear-env_agent_dir)
- [ENV_SESSION_DIR](#gear-env_session_dir)
- [FS_WATCH_RETRY_DELAY_MS](#gear-fs_watch_retry_delay_ms)
- [theme](#gear-theme)
- [DEFAULT_MAX_LINES](#gear-default_max_lines)
- [DEFAULT_MAX_BYTES](#gear-default_max_bytes)
- [GREP_MAX_LINE_LENGTH](#gear-grep_max_line_length)
- [COMPACTION_SUMMARY_PREFIX](#gear-compaction_summary_prefix)
- [COMPACTION_SUMMARY_SUFFIX](#gear-compaction_summary_suffix)
- [BRANCH_SUMMARY_PREFIX](#gear-branch_summary_prefix)
- [BRANCH_SUMMARY_SUFFIX](#gear-branch_summary_suffix)
- [CURRENT_SESSION_VERSION](#gear-current_session_version)
- [SUMMARIZATION_SYSTEM_PROMPT](#gear-summarization_system_prompt)
- [DEFAULT_COMPACTION_SETTINGS](#gear-default_compaction_settings)
- [KEYBINDINGS](#gear-keybindings)
- [BUILT_IN_PROVIDER_DISPLAY_NAMES](#gear-built_in_provider_display_names)
- [clearApiKeyCache](#gear-clearapikeycache)
- [BUILTIN_SLASH_COMMANDS](#gear-builtin_slash_commands)
- [allToolNames](#gear-alltoolnames)
- [DEFAULT_THINKING_LEVEL](#gear-default_thinking_level)
- [defaultModelPerProvider](#gear-defaultmodelperprovider)

### :gear: isBunBinary

检测当前是否以 Bun 编译二进制形式运行。
Bun 编译后的二进制文件中 `import.meta.url` 会包含 Bun 虚拟文件系统路径标识：
`"$bunfs"`、`"~BUN"` 或 `"%7EBUN"`

| Constant | Type |
| ---------- | ---------- |
| `isBunBinary` | `boolean` |

### :gear: isBunRuntime

检测当前运行时是否为 Bun（包括编译二进制和 `bun run` 两种场景）

| Constant | Type |
| ---------- | ---------- |
| `isBunRuntime` | `boolean` |

### :gear: PACKAGE_NAME

包名，默认为 "@earendil-works/pi-coding-agent"

| Constant | Type |
| ---------- | ---------- |
| `PACKAGE_NAME` | `string` |

### :gear: APP_NAME

应用名，影响配置目录名、环境变量前缀等。可被 piConfig.name 覆盖

| Constant | Type |
| ---------- | ---------- |
| `APP_NAME` | `string` |

### :gear: APP_TITLE

应用显示标题，未自定义时使用希腊字母 π

| Constant | Type |
| ---------- | ---------- |
| `APP_TITLE` | `string` |

### :gear: CONFIG_DIR_NAME

配置目录名，默认为 ".pi"（即 ~/.pi/）

| Constant | Type |
| ---------- | ---------- |
| `CONFIG_DIR_NAME` | `string` |

### :gear: VERSION

当前版本号，从 package.json 读取，兜底为 "0.0.0"

| Constant | Type |
| ---------- | ---------- |
| `VERSION` | `string` |

### :gear: ENV_AGENT_DIR

| Constant | Type |
| ---------- | ---------- |
| `ENV_AGENT_DIR` | `string` |

### :gear: ENV_SESSION_DIR

会话目录的环境变量名，用于覆盖默认的会话存储位置

| Constant | Type |
| ---------- | ---------- |
| `ENV_SESSION_DIR` | `string` |

### :gear: FS_WATCH_RETRY_DELAY_MS

| Constant | Type |
| ---------- | ---------- |
| `FS_WATCH_RETRY_DELAY_MS` | `5000` |

### :gear: theme

| Constant | Type |
| ---------- | ---------- |
| `theme` | `Theme` |

### :gear: DEFAULT_MAX_LINES

| Constant | Type |
| ---------- | ---------- |
| `DEFAULT_MAX_LINES` | `2000` |

### :gear: DEFAULT_MAX_BYTES

| Constant | Type |
| ---------- | ---------- |
| `DEFAULT_MAX_BYTES` | `number` |

### :gear: GREP_MAX_LINE_LENGTH

| Constant | Type |
| ---------- | ---------- |
| `GREP_MAX_LINE_LENGTH` | `500` |

### :gear: COMPACTION_SUMMARY_PREFIX

| Constant | Type |
| ---------- | ---------- |
| `COMPACTION_SUMMARY_PREFIX` | `"The conversation history before this point was compacted into the following summary:\n\n<summary>\n"` |

### :gear: COMPACTION_SUMMARY_SUFFIX

| Constant | Type |
| ---------- | ---------- |
| `COMPACTION_SUMMARY_SUFFIX` | `"\n</summary>"` |

### :gear: BRANCH_SUMMARY_PREFIX

| Constant | Type |
| ---------- | ---------- |
| `BRANCH_SUMMARY_PREFIX` | `"The following is a summary of a branch that this conversation came back from:\n\n<summary>\n"` |

### :gear: BRANCH_SUMMARY_SUFFIX

| Constant | Type |
| ---------- | ---------- |
| `BRANCH_SUMMARY_SUFFIX` | `"</summary>"` |

### :gear: CURRENT_SESSION_VERSION

| Constant | Type |
| ---------- | ---------- |
| `CURRENT_SESSION_VERSION` | `3` |

### :gear: SUMMARIZATION_SYSTEM_PROMPT

| Constant | Type |
| ---------- | ---------- |
| `SUMMARIZATION_SYSTEM_PROMPT` | `"You are a context summarization assistant. Your task is to read a conversation between a user and an AI coding assistant, then produce a structured summary following the exact format specified.\n\nDo NOT continue the conversation. Do NOT respond to any questions in the conversation. ONLY output the structured summa...` |

### :gear: DEFAULT_COMPACTION_SETTINGS

| Constant | Type |
| ---------- | ---------- |
| `DEFAULT_COMPACTION_SETTINGS` | `CompactionSettings` |

### :gear: KEYBINDINGS

| Constant | Type |
| ---------- | ---------- |
| `KEYBINDINGS` | `{ readonly "app.interrupt": { readonly defaultKeys: "escape"; readonly description: "Cancel or abort"; }; readonly "app.clear": { readonly defaultKeys: "ctrl+c"; readonly description: "Clear editor"; }; ... 69 more ...; readonly "tui.select.cancel": { ...; }; }` |

### :gear: BUILT_IN_PROVIDER_DISPLAY_NAMES

| Constant | Type |
| ---------- | ---------- |
| `BUILT_IN_PROVIDER_DISPLAY_NAMES` | `Record<string, string>` |

### :gear: clearApiKeyCache

Clear the config value command cache. Exported for testing.

| Constant | Type |
| ---------- | ---------- |
| `clearApiKeyCache` | `() => void` |

### :gear: BUILTIN_SLASH_COMMANDS

| Constant | Type |
| ---------- | ---------- |
| `BUILTIN_SLASH_COMMANDS` | `readonly BuiltinSlashCommand[]` |

### :gear: allToolNames

所有内置工具名称的集合，用于快速判断某个名称是否合法

| Constant | Type |
| ---------- | ---------- |
| `allToolNames` | `Set<ToolName>` |

### :gear: DEFAULT_THINKING_LEVEL

| Constant | Type |
| ---------- | ---------- |
| `DEFAULT_THINKING_LEVEL` | `ThinkingLevel` |

### :gear: defaultModelPerProvider

各已知提供商的默认模型 ID

当用户指定了提供商但未指定模型，或需要回退到默认模型时使用此映射。
更新此映射时需同步更新 `packages/ai/scripts/generate-models.ts`。

| Constant | Type |
| ---------- | ---------- |
| `defaultModelPerProvider` | `Record<KnownProvider, string>` |


## :factory: FileSettingsStorage

### Methods

- [withLock](#gear-withlock)

#### :gear: withLock

| Method | Type |
| ---------- | ---------- |
| `withLock` | `(scope: SettingsScope, fn: (current: string or undefined) => string or undefined) => void` |

## :factory: InMemorySettingsStorage

### Methods

- [withLock](#gear-withlock)

#### :gear: withLock

| Method | Type |
| ---------- | ---------- |
| `withLock` | `(scope: SettingsScope, fn: (current: string or undefined) => string or undefined) => void` |

## :factory: SettingsManager

### Static Methods

- [create](#gear-create)
- [fromStorage](#gear-fromstorage)
- [inMemory](#gear-inmemory)

#### :gear: create

Create a SettingsManager that loads from files

| Method | Type |
| ---------- | ---------- |
| `create` | `(cwd: string, agentDir?: string) => SettingsManager` |

#### :gear: fromStorage

Create a SettingsManager from an arbitrary storage backend

| Method | Type |
| ---------- | ---------- |
| `fromStorage` | `(storage: SettingsStorage) => SettingsManager` |

#### :gear: inMemory

Create an in-memory SettingsManager (no file I/O)

| Method | Type |
| ---------- | ---------- |
| `inMemory` | `(settings?: Partial<Settings>) => SettingsManager` |

### Methods

- [getGlobalSettings](#gear-getglobalsettings)
- [getProjectSettings](#gear-getprojectsettings)
- [applyOverrides](#gear-applyoverrides)
- [drainErrors](#gear-drainerrors)
- [getLastChangelogVersion](#gear-getlastchangelogversion)
- [setLastChangelogVersion](#gear-setlastchangelogversion)
- [getSessionDir](#gear-getsessiondir)
- [getDefaultProvider](#gear-getdefaultprovider)
- [getDefaultModel](#gear-getdefaultmodel)
- [setDefaultProvider](#gear-setdefaultprovider)
- [setDefaultModel](#gear-setdefaultmodel)
- [setDefaultModelAndProvider](#gear-setdefaultmodelandprovider)
- [getSteeringMode](#gear-getsteeringmode)
- [setSteeringMode](#gear-setsteeringmode)
- [getFollowUpMode](#gear-getfollowupmode)
- [setFollowUpMode](#gear-setfollowupmode)
- [getTheme](#gear-gettheme)
- [setTheme](#gear-settheme)
- [getDefaultThinkingLevel](#gear-getdefaultthinkinglevel)
- [setDefaultThinkingLevel](#gear-setdefaultthinkinglevel)
- [getTransport](#gear-gettransport)
- [setTransport](#gear-settransport)
- [getCompactionEnabled](#gear-getcompactionenabled)
- [setCompactionEnabled](#gear-setcompactionenabled)
- [getCompactionReserveTokens](#gear-getcompactionreservetokens)
- [getCompactionKeepRecentTokens](#gear-getcompactionkeeprecenttokens)
- [getCompactionSettings](#gear-getcompactionsettings)
- [getBranchSummarySettings](#gear-getbranchsummarysettings)
- [getBranchSummarySkipPrompt](#gear-getbranchsummaryskipprompt)
- [getRetryEnabled](#gear-getretryenabled)
- [setRetryEnabled](#gear-setretryenabled)
- [getRetrySettings](#gear-getretrysettings)
- [getProviderRetrySettings](#gear-getproviderretrysettings)
- [getHideThinkingBlock](#gear-gethidethinkingblock)
- [setHideThinkingBlock](#gear-sethidethinkingblock)
- [getShellPath](#gear-getshellpath)
- [setShellPath](#gear-setshellpath)
- [getQuietStartup](#gear-getquietstartup)
- [setQuietStartup](#gear-setquietstartup)
- [getShellCommandPrefix](#gear-getshellcommandprefix)
- [setShellCommandPrefix](#gear-setshellcommandprefix)
- [getNpmCommand](#gear-getnpmcommand)
- [setNpmCommand](#gear-setnpmcommand)
- [getCollapseChangelog](#gear-getcollapsechangelog)
- [setCollapseChangelog](#gear-setcollapsechangelog)
- [getEnableInstallTelemetry](#gear-getenableinstalltelemetry)
- [setEnableInstallTelemetry](#gear-setenableinstalltelemetry)
- [getPackages](#gear-getpackages)
- [setPackages](#gear-setpackages)
- [setProjectPackages](#gear-setprojectpackages)
- [getExtensionPaths](#gear-getextensionpaths)
- [setExtensionPaths](#gear-setextensionpaths)
- [setProjectExtensionPaths](#gear-setprojectextensionpaths)
- [getSkillPaths](#gear-getskillpaths)
- [setSkillPaths](#gear-setskillpaths)
- [setProjectSkillPaths](#gear-setprojectskillpaths)
- [getPromptTemplatePaths](#gear-getprompttemplatepaths)
- [setPromptTemplatePaths](#gear-setprompttemplatepaths)
- [setProjectPromptTemplatePaths](#gear-setprojectprompttemplatepaths)
- [getThemePaths](#gear-getthemepaths)
- [setThemePaths](#gear-setthemepaths)
- [setProjectThemePaths](#gear-setprojectthemepaths)
- [getEnableSkillCommands](#gear-getenableskillcommands)
- [setEnableSkillCommands](#gear-setenableskillcommands)
- [getThinkingBudgets](#gear-getthinkingbudgets)
- [getShowImages](#gear-getshowimages)
- [setShowImages](#gear-setshowimages)
- [getImageWidthCells](#gear-getimagewidthcells)
- [setImageWidthCells](#gear-setimagewidthcells)
- [getClearOnShrink](#gear-getclearonshrink)
- [setClearOnShrink](#gear-setclearonshrink)
- [getShowTerminalProgress](#gear-getshowterminalprogress)
- [setShowTerminalProgress](#gear-setshowterminalprogress)
- [getImageAutoResize](#gear-getimageautoresize)
- [setImageAutoResize](#gear-setimageautoresize)
- [getBlockImages](#gear-getblockimages)
- [setBlockImages](#gear-setblockimages)
- [getEnabledModels](#gear-getenabledmodels)
- [setEnabledModels](#gear-setenabledmodels)
- [getDoubleEscapeAction](#gear-getdoubleescapeaction)
- [setDoubleEscapeAction](#gear-setdoubleescapeaction)
- [getTreeFilterMode](#gear-gettreefiltermode)
- [setTreeFilterMode](#gear-settreefiltermode)
- [getShowHardwareCursor](#gear-getshowhardwarecursor)
- [setShowHardwareCursor](#gear-setshowhardwarecursor)
- [getEditorPaddingX](#gear-geteditorpaddingx)
- [setEditorPaddingX](#gear-seteditorpaddingx)
- [getAutocompleteMaxVisible](#gear-getautocompletemaxvisible)
- [setAutocompleteMaxVisible](#gear-setautocompletemaxvisible)
- [getCodeBlockIndent](#gear-getcodeblockindent)
- [getWarnings](#gear-getwarnings)
- [setWarnings](#gear-setwarnings)

#### :gear: getGlobalSettings

| Method | Type |
| ---------- | ---------- |
| `getGlobalSettings` | `() => Settings` |

#### :gear: getProjectSettings

| Method | Type |
| ---------- | ---------- |
| `getProjectSettings` | `() => Settings` |

#### :gear: applyOverrides

Apply additional overrides on top of current settings

| Method | Type |
| ---------- | ---------- |
| `applyOverrides` | `(overrides: Partial<Settings>) => void` |

#### :gear: drainErrors

| Method | Type |
| ---------- | ---------- |
| `drainErrors` | `() => SettingsError[]` |

#### :gear: getLastChangelogVersion

| Method | Type |
| ---------- | ---------- |
| `getLastChangelogVersion` | `() => string or undefined` |

#### :gear: setLastChangelogVersion

| Method | Type |
| ---------- | ---------- |
| `setLastChangelogVersion` | `(version: string) => void` |

#### :gear: getSessionDir

| Method | Type |
| ---------- | ---------- |
| `getSessionDir` | `() => string or undefined` |

#### :gear: getDefaultProvider

| Method | Type |
| ---------- | ---------- |
| `getDefaultProvider` | `() => string or undefined` |

#### :gear: getDefaultModel

| Method | Type |
| ---------- | ---------- |
| `getDefaultModel` | `() => string or undefined` |

#### :gear: setDefaultProvider

| Method | Type |
| ---------- | ---------- |
| `setDefaultProvider` | `(provider: string) => void` |

#### :gear: setDefaultModel

| Method | Type |
| ---------- | ---------- |
| `setDefaultModel` | `(modelId: string) => void` |

#### :gear: setDefaultModelAndProvider

| Method | Type |
| ---------- | ---------- |
| `setDefaultModelAndProvider` | `(provider: string, modelId: string) => void` |

#### :gear: getSteeringMode

| Method | Type |
| ---------- | ---------- |
| `getSteeringMode` | `() => "all" or "one-at-a-time"` |

#### :gear: setSteeringMode

| Method | Type |
| ---------- | ---------- |
| `setSteeringMode` | `(mode: "all" or "one-at-a-time") => void` |

#### :gear: getFollowUpMode

| Method | Type |
| ---------- | ---------- |
| `getFollowUpMode` | `() => "all" or "one-at-a-time"` |

#### :gear: setFollowUpMode

| Method | Type |
| ---------- | ---------- |
| `setFollowUpMode` | `(mode: "all" or "one-at-a-time") => void` |

#### :gear: getTheme

| Method | Type |
| ---------- | ---------- |
| `getTheme` | `() => string or undefined` |

#### :gear: setTheme

| Method | Type |
| ---------- | ---------- |
| `setTheme` | `(theme: string) => void` |

#### :gear: getDefaultThinkingLevel

| Method | Type |
| ---------- | ---------- |
| `getDefaultThinkingLevel` | `() => "off" or "minimal" or "low" or "medium" or "high" or "xhigh" or undefined` |

#### :gear: setDefaultThinkingLevel

| Method | Type |
| ---------- | ---------- |
| `setDefaultThinkingLevel` | `(level: "off" or "minimal" or "low" or "medium" or "high" or "xhigh") => void` |

#### :gear: getTransport

| Method | Type |
| ---------- | ---------- |
| `getTransport` | `() => Transport` |

#### :gear: setTransport

| Method | Type |
| ---------- | ---------- |
| `setTransport` | `(transport: Transport) => void` |

#### :gear: getCompactionEnabled

| Method | Type |
| ---------- | ---------- |
| `getCompactionEnabled` | `() => boolean` |

#### :gear: setCompactionEnabled

| Method | Type |
| ---------- | ---------- |
| `setCompactionEnabled` | `(enabled: boolean) => void` |

#### :gear: getCompactionReserveTokens

| Method | Type |
| ---------- | ---------- |
| `getCompactionReserveTokens` | `() => number` |

#### :gear: getCompactionKeepRecentTokens

| Method | Type |
| ---------- | ---------- |
| `getCompactionKeepRecentTokens` | `() => number` |

#### :gear: getCompactionSettings

| Method | Type |
| ---------- | ---------- |
| `getCompactionSettings` | `() => { enabled: boolean; reserveTokens: number; keepRecentTokens: number; }` |

#### :gear: getBranchSummarySettings

| Method | Type |
| ---------- | ---------- |
| `getBranchSummarySettings` | `() => { reserveTokens: number; skipPrompt: boolean; }` |

#### :gear: getBranchSummarySkipPrompt

| Method | Type |
| ---------- | ---------- |
| `getBranchSummarySkipPrompt` | `() => boolean` |

#### :gear: getRetryEnabled

| Method | Type |
| ---------- | ---------- |
| `getRetryEnabled` | `() => boolean` |

#### :gear: setRetryEnabled

| Method | Type |
| ---------- | ---------- |
| `setRetryEnabled` | `(enabled: boolean) => void` |

#### :gear: getRetrySettings

| Method | Type |
| ---------- | ---------- |
| `getRetrySettings` | `() => { enabled: boolean; maxRetries: number; baseDelayMs: number; }` |

#### :gear: getProviderRetrySettings

| Method | Type |
| ---------- | ---------- |
| `getProviderRetrySettings` | `() => { timeoutMs?: number or undefined; maxRetries?: number or undefined; maxRetryDelayMs: number; }` |

#### :gear: getHideThinkingBlock

| Method | Type |
| ---------- | ---------- |
| `getHideThinkingBlock` | `() => boolean` |

#### :gear: setHideThinkingBlock

| Method | Type |
| ---------- | ---------- |
| `setHideThinkingBlock` | `(hide: boolean) => void` |

#### :gear: getShellPath

| Method | Type |
| ---------- | ---------- |
| `getShellPath` | `() => string or undefined` |

#### :gear: setShellPath

| Method | Type |
| ---------- | ---------- |
| `setShellPath` | `(path: string or undefined) => void` |

#### :gear: getQuietStartup

| Method | Type |
| ---------- | ---------- |
| `getQuietStartup` | `() => boolean` |

#### :gear: setQuietStartup

| Method | Type |
| ---------- | ---------- |
| `setQuietStartup` | `(quiet: boolean) => void` |

#### :gear: getShellCommandPrefix

| Method | Type |
| ---------- | ---------- |
| `getShellCommandPrefix` | `() => string or undefined` |

#### :gear: setShellCommandPrefix

| Method | Type |
| ---------- | ---------- |
| `setShellCommandPrefix` | `(prefix: string or undefined) => void` |

#### :gear: getNpmCommand

| Method | Type |
| ---------- | ---------- |
| `getNpmCommand` | `() => string[] or undefined` |

#### :gear: setNpmCommand

| Method | Type |
| ---------- | ---------- |
| `setNpmCommand` | `(command: string[] or undefined) => void` |

#### :gear: getCollapseChangelog

| Method | Type |
| ---------- | ---------- |
| `getCollapseChangelog` | `() => boolean` |

#### :gear: setCollapseChangelog

| Method | Type |
| ---------- | ---------- |
| `setCollapseChangelog` | `(collapse: boolean) => void` |

#### :gear: getEnableInstallTelemetry

| Method | Type |
| ---------- | ---------- |
| `getEnableInstallTelemetry` | `() => boolean` |

#### :gear: setEnableInstallTelemetry

| Method | Type |
| ---------- | ---------- |
| `setEnableInstallTelemetry` | `(enabled: boolean) => void` |

#### :gear: getPackages

| Method | Type |
| ---------- | ---------- |
| `getPackages` | `() => PackageSource[]` |

#### :gear: setPackages

| Method | Type |
| ---------- | ---------- |
| `setPackages` | `(packages: PackageSource[]) => void` |

#### :gear: setProjectPackages

| Method | Type |
| ---------- | ---------- |
| `setProjectPackages` | `(packages: PackageSource[]) => void` |

#### :gear: getExtensionPaths

| Method | Type |
| ---------- | ---------- |
| `getExtensionPaths` | `() => string[]` |

#### :gear: setExtensionPaths

| Method | Type |
| ---------- | ---------- |
| `setExtensionPaths` | `(paths: string[]) => void` |

#### :gear: setProjectExtensionPaths

| Method | Type |
| ---------- | ---------- |
| `setProjectExtensionPaths` | `(paths: string[]) => void` |

#### :gear: getSkillPaths

| Method | Type |
| ---------- | ---------- |
| `getSkillPaths` | `() => string[]` |

#### :gear: setSkillPaths

| Method | Type |
| ---------- | ---------- |
| `setSkillPaths` | `(paths: string[]) => void` |

#### :gear: setProjectSkillPaths

| Method | Type |
| ---------- | ---------- |
| `setProjectSkillPaths` | `(paths: string[]) => void` |

#### :gear: getPromptTemplatePaths

| Method | Type |
| ---------- | ---------- |
| `getPromptTemplatePaths` | `() => string[]` |

#### :gear: setPromptTemplatePaths

| Method | Type |
| ---------- | ---------- |
| `setPromptTemplatePaths` | `(paths: string[]) => void` |

#### :gear: setProjectPromptTemplatePaths

| Method | Type |
| ---------- | ---------- |
| `setProjectPromptTemplatePaths` | `(paths: string[]) => void` |

#### :gear: getThemePaths

| Method | Type |
| ---------- | ---------- |
| `getThemePaths` | `() => string[]` |

#### :gear: setThemePaths

| Method | Type |
| ---------- | ---------- |
| `setThemePaths` | `(paths: string[]) => void` |

#### :gear: setProjectThemePaths

| Method | Type |
| ---------- | ---------- |
| `setProjectThemePaths` | `(paths: string[]) => void` |

#### :gear: getEnableSkillCommands

| Method | Type |
| ---------- | ---------- |
| `getEnableSkillCommands` | `() => boolean` |

#### :gear: setEnableSkillCommands

| Method | Type |
| ---------- | ---------- |
| `setEnableSkillCommands` | `(enabled: boolean) => void` |

#### :gear: getThinkingBudgets

| Method | Type |
| ---------- | ---------- |
| `getThinkingBudgets` | `() => ThinkingBudgetsSettings or undefined` |

#### :gear: getShowImages

| Method | Type |
| ---------- | ---------- |
| `getShowImages` | `() => boolean` |

#### :gear: setShowImages

| Method | Type |
| ---------- | ---------- |
| `setShowImages` | `(show: boolean) => void` |

#### :gear: getImageWidthCells

| Method | Type |
| ---------- | ---------- |
| `getImageWidthCells` | `() => number` |

#### :gear: setImageWidthCells

| Method | Type |
| ---------- | ---------- |
| `setImageWidthCells` | `(width: number) => void` |

#### :gear: getClearOnShrink

| Method | Type |
| ---------- | ---------- |
| `getClearOnShrink` | `() => boolean` |

#### :gear: setClearOnShrink

| Method | Type |
| ---------- | ---------- |
| `setClearOnShrink` | `(enabled: boolean) => void` |

#### :gear: getShowTerminalProgress

| Method | Type |
| ---------- | ---------- |
| `getShowTerminalProgress` | `() => boolean` |

#### :gear: setShowTerminalProgress

| Method | Type |
| ---------- | ---------- |
| `setShowTerminalProgress` | `(enabled: boolean) => void` |

#### :gear: getImageAutoResize

| Method | Type |
| ---------- | ---------- |
| `getImageAutoResize` | `() => boolean` |

#### :gear: setImageAutoResize

| Method | Type |
| ---------- | ---------- |
| `setImageAutoResize` | `(enabled: boolean) => void` |

#### :gear: getBlockImages

| Method | Type |
| ---------- | ---------- |
| `getBlockImages` | `() => boolean` |

#### :gear: setBlockImages

| Method | Type |
| ---------- | ---------- |
| `setBlockImages` | `(blocked: boolean) => void` |

#### :gear: getEnabledModels

| Method | Type |
| ---------- | ---------- |
| `getEnabledModels` | `() => string[] or undefined` |

#### :gear: setEnabledModels

| Method | Type |
| ---------- | ---------- |
| `setEnabledModels` | `(patterns: string[] or undefined) => void` |

#### :gear: getDoubleEscapeAction

| Method | Type |
| ---------- | ---------- |
| `getDoubleEscapeAction` | `() => "fork" or "tree" or "none"` |

#### :gear: setDoubleEscapeAction

| Method | Type |
| ---------- | ---------- |
| `setDoubleEscapeAction` | `(action: "fork" or "tree" or "none") => void` |

#### :gear: getTreeFilterMode

| Method | Type |
| ---------- | ---------- |
| `getTreeFilterMode` | `() => "all" or "default" or "no-tools" or "user-only" or "labeled-only"` |

#### :gear: setTreeFilterMode

| Method | Type |
| ---------- | ---------- |
| `setTreeFilterMode` | `(mode: "all" or "default" or "no-tools" or "user-only" or "labeled-only") => void` |

#### :gear: getShowHardwareCursor

| Method | Type |
| ---------- | ---------- |
| `getShowHardwareCursor` | `() => boolean` |

#### :gear: setShowHardwareCursor

| Method | Type |
| ---------- | ---------- |
| `setShowHardwareCursor` | `(enabled: boolean) => void` |

#### :gear: getEditorPaddingX

| Method | Type |
| ---------- | ---------- |
| `getEditorPaddingX` | `() => number` |

#### :gear: setEditorPaddingX

| Method | Type |
| ---------- | ---------- |
| `setEditorPaddingX` | `(padding: number) => void` |

#### :gear: getAutocompleteMaxVisible

| Method | Type |
| ---------- | ---------- |
| `getAutocompleteMaxVisible` | `() => number` |

#### :gear: setAutocompleteMaxVisible

| Method | Type |
| ---------- | ---------- |
| `setAutocompleteMaxVisible` | `(maxVisible: number) => void` |

#### :gear: getCodeBlockIndent

| Method | Type |
| ---------- | ---------- |
| `getCodeBlockIndent` | `() => string` |

#### :gear: getWarnings

| Method | Type |
| ---------- | ---------- |
| `getWarnings` | `() => WarningSettings` |

#### :gear: setWarnings

| Method | Type |
| ---------- | ---------- |
| `setWarnings` | `(warnings: WarningSettings) => void` |

## :factory: DefaultPackageManager

### Methods

- [setProgressCallback](#gear-setprogresscallback)
- [addSourceToSettings](#gear-addsourcetosettings)
- [removeSourceFromSettings](#gear-removesourcefromsettings)
- [getInstalledPath](#gear-getinstalledpath)
- [listConfiguredPackages](#gear-listconfiguredpackages)

#### :gear: setProgressCallback

| Method | Type |
| ---------- | ---------- |
| `setProgressCallback` | `(callback: ProgressCallback or undefined) => void` |

#### :gear: addSourceToSettings

| Method | Type |
| ---------- | ---------- |
| `addSourceToSettings` | `(source: string, options?: { local?: boolean or undefined; } or undefined) => boolean` |

#### :gear: removeSourceFromSettings

| Method | Type |
| ---------- | ---------- |
| `removeSourceFromSettings` | `(source: string, options?: { local?: boolean or undefined; } or undefined) => boolean` |

#### :gear: getInstalledPath

| Method | Type |
| ---------- | ---------- |
| `getInstalledPath` | `(source: string, scope: "project" or "user") => string or undefined` |

#### :gear: listConfiguredPackages

| Method | Type |
| ---------- | ---------- |
| `listConfiguredPackages` | `() => ConfiguredPackage[]` |

## :factory: Theme

### Methods

- [fg](#gear-fg)
- [bg](#gear-bg)
- [bold](#gear-bold)
- [italic](#gear-italic)
- [underline](#gear-underline)
- [inverse](#gear-inverse)
- [strikethrough](#gear-strikethrough)
- [getFgAnsi](#gear-getfgansi)
- [getBgAnsi](#gear-getbgansi)
- [getColorMode](#gear-getcolormode)
- [getThinkingBorderColor](#gear-getthinkingbordercolor)
- [getBashModeBorderColor](#gear-getbashmodebordercolor)

#### :gear: fg

| Method | Type |
| ---------- | ---------- |
| `fg` | `(color: ThemeColor, text: string) => string` |

#### :gear: bg

| Method | Type |
| ---------- | ---------- |
| `bg` | `(color: ThemeBg, text: string) => string` |

#### :gear: bold

| Method | Type |
| ---------- | ---------- |
| `bold` | `(text: string) => string` |

#### :gear: italic

| Method | Type |
| ---------- | ---------- |
| `italic` | `(text: string) => string` |

#### :gear: underline

| Method | Type |
| ---------- | ---------- |
| `underline` | `(text: string) => string` |

#### :gear: inverse

| Method | Type |
| ---------- | ---------- |
| `inverse` | `(text: string) => string` |

#### :gear: strikethrough

| Method | Type |
| ---------- | ---------- |
| `strikethrough` | `(text: string) => string` |

#### :gear: getFgAnsi

| Method | Type |
| ---------- | ---------- |
| `getFgAnsi` | `(color: ThemeColor) => string` |

#### :gear: getBgAnsi

| Method | Type |
| ---------- | ---------- |
| `getBgAnsi` | `(color: ThemeBg) => string` |

#### :gear: getColorMode

| Method | Type |
| ---------- | ---------- |
| `getColorMode` | `() => ColorMode` |

#### :gear: getThinkingBorderColor

| Method | Type |
| ---------- | ---------- |
| `getThinkingBorderColor` | `(level: "off" or "minimal" or "low" or "medium" or "high" or "xhigh") => (str: string) => string` |

#### :gear: getBashModeBorderColor

| Method | Type |
| ---------- | ---------- |
| `getBashModeBorderColor` | `() => (str: string) => string` |

### Properties

- [sourceInfo](#gear-sourceinfo)

#### :gear: sourceInfo

| Property | Type |
| ---------- | ---------- |
| `sourceInfo` | `SourceInfo or undefined` |

## :factory: OutputAccumulator

Incrementally tracks streaming output with bounded memory.

Appends decode chunks with a streaming UTF-8 decoder, keeps only a decoded
tail for display snapshots, and opens a temp file when the full output needs
to be preserved.

### Methods

- [append](#gear-append)
- [finish](#gear-finish)
- [snapshot](#gear-snapshot)
- [getLastLineBytes](#gear-getlastlinebytes)

#### :gear: append

| Method | Type |
| ---------- | ---------- |
| `append` | `(data: Buffer<ArrayBufferLike>) => void` |

#### :gear: finish

| Method | Type |
| ---------- | ---------- |
| `finish` | `() => void` |

#### :gear: snapshot

| Method | Type |
| ---------- | ---------- |
| `snapshot` | `(options?: { persistIfTruncated?: boolean or undefined; }) => OutputSnapshot` |

#### :gear: getLastLineBytes

| Method | Type |
| ---------- | ---------- |
| `getLastLineBytes` | `() => number` |

## :factory: SessionManager

Manages conversation sessions as append-only trees stored in JSONL files.

Each session entry has an id and parentId forming a tree structure. The "leaf"
pointer tracks the current position. Appending creates a child of the current leaf.
Branching moves the leaf to an earlier entry, allowing new branches without
modifying history.

Use buildSessionContext() to get the resolved message list for the LLM, which
handles compaction summaries and follows the path from root to current leaf.

### Static Methods

- [create](#gear-create)
- [open](#gear-open)
- [continueRecent](#gear-continuerecent)
- [inMemory](#gear-inmemory)
- [forkFrom](#gear-forkfrom)

#### :gear: create

Create a new session.

| Method | Type |
| ---------- | ---------- |
| `create` | `(cwd: string, sessionDir?: string or undefined) => SessionManager` |

Parameters:

* `cwd`: Working directory (stored in session header)
* `sessionDir`: Optional session directory. If omitted, uses default (~/.pi/agent/sessions/<encoded-cwd>/).


#### :gear: open

Open a specific session file.

| Method | Type |
| ---------- | ---------- |
| `open` | `(path: string, sessionDir?: string or undefined, cwdOverride?: string or undefined) => SessionManager` |

Parameters:

* `path`: Path to session file
* `sessionDir`: Optional session directory for /new or /branch. If omitted, derives from file's parent.
* `cwdOverride`: Optional cwd override instead of the session header cwd.


#### :gear: continueRecent

Continue the most recent session, or create new if none.

| Method | Type |
| ---------- | ---------- |
| `continueRecent` | `(cwd: string, sessionDir?: string or undefined) => SessionManager` |

Parameters:

* `cwd`: Working directory
* `sessionDir`: Optional session directory. If omitted, uses default (~/.pi/agent/sessions/<encoded-cwd>/).


#### :gear: inMemory

Create an in-memory session (no file persistence)

| Method | Type |
| ---------- | ---------- |
| `inMemory` | `(cwd?: string) => SessionManager` |

#### :gear: forkFrom

Fork a session from another project directory into the current project.
Creates a new session in the target cwd with the full history from the source session.

| Method | Type |
| ---------- | ---------- |
| `forkFrom` | `(sourcePath: string, targetCwd: string, sessionDir?: string or undefined) => SessionManager` |

Parameters:

* `sourcePath`: Path to the source session file
* `targetCwd`: Target working directory (where the new session will be stored)
* `sessionDir`: Optional session directory. If omitted, uses default for targetCwd.


### Methods

- [setSessionFile](#gear-setsessionfile)
- [newSession](#gear-newsession)
- [isPersisted](#gear-ispersisted)
- [getCwd](#gear-getcwd)
- [getSessionDir](#gear-getsessiondir)
- [getSessionId](#gear-getsessionid)
- [getSessionFile](#gear-getsessionfile)
- [_persist](#gear-_persist)
- [appendMessage](#gear-appendmessage)
- [appendThinkingLevelChange](#gear-appendthinkinglevelchange)
- [appendModelChange](#gear-appendmodelchange)
- [appendCompaction](#gear-appendcompaction)
- [appendCustomEntry](#gear-appendcustomentry)
- [appendSessionInfo](#gear-appendsessioninfo)
- [getSessionName](#gear-getsessionname)
- [appendCustomMessageEntry](#gear-appendcustommessageentry)
- [getLeafId](#gear-getleafid)
- [getLeafEntry](#gear-getleafentry)
- [getEntry](#gear-getentry)
- [getChildren](#gear-getchildren)
- [getLabel](#gear-getlabel)
- [appendLabelChange](#gear-appendlabelchange)
- [getBranch](#gear-getbranch)
- [buildSessionContext](#gear-buildsessioncontext)
- [getHeader](#gear-getheader)
- [getEntries](#gear-getentries)
- [getTree](#gear-gettree)
- [branch](#gear-branch)
- [resetLeaf](#gear-resetleaf)
- [branchWithSummary](#gear-branchwithsummary)
- [createBranchedSession](#gear-createbranchedsession)

#### :gear: setSessionFile

Switch to a different session file (used for resume and branching)

| Method | Type |
| ---------- | ---------- |
| `setSessionFile` | `(sessionFile: string) => void` |

#### :gear: newSession

| Method | Type |
| ---------- | ---------- |
| `newSession` | `(options?: NewSessionOptions or undefined) => string or undefined` |

#### :gear: isPersisted

| Method | Type |
| ---------- | ---------- |
| `isPersisted` | `() => boolean` |

#### :gear: getCwd

| Method | Type |
| ---------- | ---------- |
| `getCwd` | `() => string` |

#### :gear: getSessionDir

| Method | Type |
| ---------- | ---------- |
| `getSessionDir` | `() => string` |

#### :gear: getSessionId

| Method | Type |
| ---------- | ---------- |
| `getSessionId` | `() => string` |

#### :gear: getSessionFile

| Method | Type |
| ---------- | ---------- |
| `getSessionFile` | `() => string or undefined` |

#### :gear: _persist

| Method | Type |
| ---------- | ---------- |
| `_persist` | `(entry: SessionEntry) => void` |

#### :gear: appendMessage

Append a message as child of current leaf, then advance leaf. Returns entry id.
Does not allow writing CompactionSummaryMessage and BranchSummaryMessage directly.
Reason: we want these to be top-level entries in the session, not message session entries,
so it is easier to find them.
These need to be appended via appendCompaction() and appendBranchSummary() methods.

| Method | Type |
| ---------- | ---------- |
| `appendMessage` | `(message: BashExecutionMessage or CustomMessage<unknown> or Message) => string` |

#### :gear: appendThinkingLevelChange

Append a thinking level change as child of current leaf, then advance leaf. Returns entry id.

| Method | Type |
| ---------- | ---------- |
| `appendThinkingLevelChange` | `(thinkingLevel: string) => string` |

#### :gear: appendModelChange

Append a model change as child of current leaf, then advance leaf. Returns entry id.

| Method | Type |
| ---------- | ---------- |
| `appendModelChange` | `(provider: string, modelId: string) => string` |

#### :gear: appendCompaction

Append a compaction summary as child of current leaf, then advance leaf. Returns entry id.

| Method | Type |
| ---------- | ---------- |
| `appendCompaction` | `<T = unknown>(summary: string, firstKeptEntryId: string, tokensBefore: number, details?: T or undefined, fromHook?: boolean or undefined) => string` |

#### :gear: appendCustomEntry

Append a custom entry (for extensions) as child of current leaf, then advance leaf. Returns entry id.

| Method | Type |
| ---------- | ---------- |
| `appendCustomEntry` | `(customType: string, data?: unknown) => string` |

#### :gear: appendSessionInfo

Append a session info entry (e.g., display name). Returns entry id.

| Method | Type |
| ---------- | ---------- |
| `appendSessionInfo` | `(name: string) => string` |

#### :gear: getSessionName

Get the current session name from the latest session_info entry, if any.

| Method | Type |
| ---------- | ---------- |
| `getSessionName` | `() => string or undefined` |

#### :gear: appendCustomMessageEntry

Append a custom message entry (for extensions) that participates in LLM context.

| Method | Type |
| ---------- | ---------- |
| `appendCustomMessageEntry` | `<T = unknown>(customType: string, content: string or (TextContent or ImageContent)[], display: boolean, details?: T or undefined) => string` |

Parameters:

* `customType`: Extension identifier for filtering on reload
* `content`: Message content (string or TextContent/ImageContent array)
* `display`: Whether to show in TUI (true = styled display, false = hidden)
* `details`: Optional extension-specific metadata (not sent to LLM)


Returns:

Entry id

#### :gear: getLeafId

| Method | Type |
| ---------- | ---------- |
| `getLeafId` | `() => string or null` |

#### :gear: getLeafEntry

| Method | Type |
| ---------- | ---------- |
| `getLeafEntry` | `() => SessionEntry or undefined` |

#### :gear: getEntry

| Method | Type |
| ---------- | ---------- |
| `getEntry` | `(id: string) => SessionEntry or undefined` |

#### :gear: getChildren

Get all direct children of an entry.

| Method | Type |
| ---------- | ---------- |
| `getChildren` | `(parentId: string) => SessionEntry[]` |

#### :gear: getLabel

Get the label for an entry, if any.

| Method | Type |
| ---------- | ---------- |
| `getLabel` | `(id: string) => string or undefined` |

#### :gear: appendLabelChange

Set or clear a label on an entry.
Labels are user-defined markers for bookmarking/navigation.
Pass undefined or empty string to clear the label.

| Method | Type |
| ---------- | ---------- |
| `appendLabelChange` | `(targetId: string, label: string or undefined) => string` |

#### :gear: getBranch

Walk from entry to root, returning all entries in path order.
Includes all entry types (messages, compaction, model changes, etc.).
Use buildSessionContext() to get the resolved messages for the LLM.

| Method | Type |
| ---------- | ---------- |
| `getBranch` | `(fromId?: string or undefined) => SessionEntry[]` |

#### :gear: buildSessionContext

Build the session context (what gets sent to the LLM).
Uses tree traversal from current leaf.

| Method | Type |
| ---------- | ---------- |
| `buildSessionContext` | `() => SessionContext` |

#### :gear: getHeader

Get session header.

| Method | Type |
| ---------- | ---------- |
| `getHeader` | `() => SessionHeader or null` |

#### :gear: getEntries

Get all session entries (excludes header). Returns a shallow copy.
The session is append-only: use appendXXX() to add entries, branch() to
change the leaf pointer. Entries cannot be modified or deleted.

| Method | Type |
| ---------- | ---------- |
| `getEntries` | `() => SessionEntry[]` |

#### :gear: getTree

Get the session as a tree structure. Returns a shallow defensive copy of all entries.
A well-formed session has exactly one root (first entry with parentId === null).
Orphaned entries (broken parent chain) are also returned as roots.

| Method | Type |
| ---------- | ---------- |
| `getTree` | `() => SessionTreeNode[]` |

#### :gear: branch

Start a new branch from an earlier entry.
Moves the leaf pointer to the specified entry. The next appendXXX() call
will create a child of that entry, forming a new branch. Existing entries
are not modified or deleted.

| Method | Type |
| ---------- | ---------- |
| `branch` | `(branchFromId: string) => void` |

#### :gear: resetLeaf

Reset the leaf pointer to null (before any entries).
The next appendXXX() call will create a new root entry (parentId = null).
Use this when navigating to re-edit the first user message.

| Method | Type |
| ---------- | ---------- |
| `resetLeaf` | `() => void` |

#### :gear: branchWithSummary

Start a new branch with a summary of the abandoned path.
Same as branch(), but also appends a branch_summary entry that captures
context from the abandoned conversation path.

| Method | Type |
| ---------- | ---------- |
| `branchWithSummary` | `(branchFromId: string or null, summary: string, details?: unknown, fromHook?: boolean or undefined) => string` |

#### :gear: createBranchedSession

Create a new session file containing only the path from root to the specified leaf.
Useful for extracting a single conversation path from a branched session.
Returns the new session file path, or undefined if not persisting.

| Method | Type |
| ---------- | ---------- |
| `createBranchedSession` | `(leafId: string) => string or undefined` |

## :factory: FooterDataProvider

Provides git branch and extension statuses - data not otherwise accessible to extensions.
Token stats, model info available via ctx.sessionManager and ctx.model.

### Methods

- [getGitBranch](#gear-getgitbranch)
- [getExtensionStatuses](#gear-getextensionstatuses)
- [onBranchChange](#gear-onbranchchange)
- [setExtensionStatus](#gear-setextensionstatus)
- [clearExtensionStatuses](#gear-clearextensionstatuses)
- [getAvailableProviderCount](#gear-getavailableprovidercount)
- [setAvailableProviderCount](#gear-setavailableprovidercount)
- [setCwd](#gear-setcwd)
- [dispose](#gear-dispose)

#### :gear: getGitBranch

Current git branch, null if not in repo, "detached" if detached HEAD

| Method | Type |
| ---------- | ---------- |
| `getGitBranch` | `() => string or null` |

#### :gear: getExtensionStatuses

Extension status texts set via ctx.ui.setStatus()

| Method | Type |
| ---------- | ---------- |
| `getExtensionStatuses` | `() => ReadonlyMap<string, string>` |

#### :gear: onBranchChange

Subscribe to git branch changes. Returns unsubscribe function.

| Method | Type |
| ---------- | ---------- |
| `onBranchChange` | `(callback: () => void) => () => void` |

#### :gear: setExtensionStatus

Internal: set extension status

| Method | Type |
| ---------- | ---------- |
| `setExtensionStatus` | `(key: string, text: string or undefined) => void` |

#### :gear: clearExtensionStatuses

Internal: clear extension statuses

| Method | Type |
| ---------- | ---------- |
| `clearExtensionStatuses` | `() => void` |

#### :gear: getAvailableProviderCount

Number of unique providers with available models (for footer display)

| Method | Type |
| ---------- | ---------- |
| `getAvailableProviderCount` | `() => number` |

#### :gear: setAvailableProviderCount

Internal: update available provider count

| Method | Type |
| ---------- | ---------- |
| `setAvailableProviderCount` | `(count: number) => void` |

#### :gear: setCwd

| Method | Type |
| ---------- | ---------- |
| `setCwd` | `(cwd: string) => void` |

#### :gear: dispose

Internal: cleanup

| Method | Type |
| ---------- | ---------- |
| `dispose` | `() => void` |

## :factory: KeybindingsManager

### Static Methods

- [create](#gear-create)

#### :gear: create

| Method | Type |
| ---------- | ---------- |
| `create` | `(agentDir?: string) => KeybindingsManager` |

### Methods

- [reload](#gear-reload)
- [getEffectiveConfig](#gear-geteffectiveconfig)

#### :gear: reload

| Method | Type |
| ---------- | ---------- |
| `reload` | `() => void` |

#### :gear: getEffectiveConfig

| Method | Type |
| ---------- | ---------- |
| `getEffectiveConfig` | `() => KeybindingsConfig` |

## :factory: FileAuthStorageBackend

### Methods

- [withLock](#gear-withlock)

#### :gear: withLock

| Method | Type |
| ---------- | ---------- |
| `withLock` | `<T>(fn: (current: string or undefined) => LockResult<T>) => T` |

## :factory: InMemoryAuthStorageBackend

### Methods

- [withLock](#gear-withlock)

#### :gear: withLock

| Method | Type |
| ---------- | ---------- |
| `withLock` | `<T>(fn: (current: string or undefined) => LockResult<T>) => T` |

## :factory: AuthStorage

Credential storage backed by a JSON file.

### Static Methods

- [create](#gear-create)
- [fromStorage](#gear-fromstorage)
- [inMemory](#gear-inmemory)

#### :gear: create

| Method | Type |
| ---------- | ---------- |
| `create` | `(authPath?: string or undefined) => AuthStorage` |

#### :gear: fromStorage

| Method | Type |
| ---------- | ---------- |
| `fromStorage` | `(storage: AuthStorageBackend) => AuthStorage` |

#### :gear: inMemory

| Method | Type |
| ---------- | ---------- |
| `inMemory` | `(data?: AuthStorageData) => AuthStorage` |

### Methods

- [setRuntimeApiKey](#gear-setruntimeapikey)
- [removeRuntimeApiKey](#gear-removeruntimeapikey)
- [setFallbackResolver](#gear-setfallbackresolver)
- [reload](#gear-reload)
- [get](#gear-get)
- [set](#gear-set)
- [remove](#gear-remove)
- [list](#gear-list)
- [has](#gear-has)
- [hasAuth](#gear-hasauth)
- [getAuthStatus](#gear-getauthstatus)
- [getAll](#gear-getall)
- [drainErrors](#gear-drainerrors)
- [logout](#gear-logout)
- [getOAuthProviders](#gear-getoauthproviders)

#### :gear: setRuntimeApiKey

Set a runtime API key override (not persisted to disk).
Used for CLI --api-key flag.

| Method | Type |
| ---------- | ---------- |
| `setRuntimeApiKey` | `(provider: string, apiKey: string) => void` |

#### :gear: removeRuntimeApiKey

Remove a runtime API key override.

| Method | Type |
| ---------- | ---------- |
| `removeRuntimeApiKey` | `(provider: string) => void` |

#### :gear: setFallbackResolver

Set a fallback resolver for API keys not found in auth.json or env vars.
Used for custom provider keys from models.json.

| Method | Type |
| ---------- | ---------- |
| `setFallbackResolver` | `(resolver: (provider: string) => string or undefined) => void` |

#### :gear: reload

Reload credentials from storage.

| Method | Type |
| ---------- | ---------- |
| `reload` | `() => void` |

#### :gear: get

Get credential for a provider.

| Method | Type |
| ---------- | ---------- |
| `get` | `(provider: string) => AuthCredential or undefined` |

#### :gear: set

Set credential for a provider.

| Method | Type |
| ---------- | ---------- |
| `set` | `(provider: string, credential: AuthCredential) => void` |

#### :gear: remove

Remove credential for a provider.

| Method | Type |
| ---------- | ---------- |
| `remove` | `(provider: string) => void` |

#### :gear: list

List all providers with credentials.

| Method | Type |
| ---------- | ---------- |
| `list` | `() => string[]` |

#### :gear: has

Check if credentials exist for a provider in auth.json.

| Method | Type |
| ---------- | ---------- |
| `has` | `(provider: string) => boolean` |

#### :gear: hasAuth

Check if any form of auth is configured for a provider.
Unlike getApiKey(), this doesn't refresh OAuth tokens.

| Method | Type |
| ---------- | ---------- |
| `hasAuth` | `(provider: string) => boolean` |

#### :gear: getAuthStatus

Return auth status without exposing credential values or refreshing tokens.

| Method | Type |
| ---------- | ---------- |
| `getAuthStatus` | `(provider: string) => AuthStatus` |

#### :gear: getAll

Get all credentials (for passing to getOAuthApiKey).

| Method | Type |
| ---------- | ---------- |
| `getAll` | `() => AuthStorageData` |

#### :gear: drainErrors

| Method | Type |
| ---------- | ---------- |
| `drainErrors` | `() => Error[]` |

#### :gear: logout

Logout from a provider.

| Method | Type |
| ---------- | ---------- |
| `logout` | `(provider: string) => void` |

#### :gear: getOAuthProviders

Get all registered OAuth providers

| Method | Type |
| ---------- | ---------- |
| `getOAuthProviders` | `() => any` |

## :factory: ModelRegistry

Model registry - loads and manages models, resolves API keys via AuthStorage.

### Static Methods

- [create](#gear-create)
- [inMemory](#gear-inmemory)

#### :gear: create

| Method | Type |
| ---------- | ---------- |
| `create` | `(authStorage: AuthStorage, modelsJsonPath?: string) => ModelRegistry` |

#### :gear: inMemory

| Method | Type |
| ---------- | ---------- |
| `inMemory` | `(authStorage: AuthStorage) => ModelRegistry` |

### Methods

- [refresh](#gear-refresh)
- [getError](#gear-geterror)
- [getAll](#gear-getall)
- [getAvailable](#gear-getavailable)
- [find](#gear-find)
- [hasConfiguredAuth](#gear-hasconfiguredauth)
- [getProviderAuthStatus](#gear-getproviderauthstatus)
- [getProviderDisplayName](#gear-getproviderdisplayname)
- [isUsingOAuth](#gear-isusingoauth)
- [registerProvider](#gear-registerprovider)
- [unregisterProvider](#gear-unregisterprovider)

#### :gear: refresh

Reload models from disk (built-in + custom from models.json).

| Method | Type |
| ---------- | ---------- |
| `refresh` | `() => void` |

#### :gear: getError

Get any error from loading models.json (undefined if no error).

| Method | Type |
| ---------- | ---------- |
| `getError` | `() => string or undefined` |

#### :gear: getAll

Get all models (built-in + custom).
If models.json had errors, returns only built-in models.

| Method | Type |
| ---------- | ---------- |
| `getAll` | `() => Model<Api>[]` |

#### :gear: getAvailable

Get only models that have auth configured.
This is a fast check that doesn't refresh OAuth tokens.

| Method | Type |
| ---------- | ---------- |
| `getAvailable` | `() => Model<Api>[]` |

#### :gear: find

Find a model by provider and ID.

| Method | Type |
| ---------- | ---------- |
| `find` | `(provider: string, modelId: string) => Model<Api> or undefined` |

#### :gear: hasConfiguredAuth

Get API key for a model.

| Method | Type |
| ---------- | ---------- |
| `hasConfiguredAuth` | `(model: Model<Api>) => boolean` |

#### :gear: getProviderAuthStatus

Return auth status for a provider, including request auth configured in models.json.
This intentionally does not execute command-backed config values.

| Method | Type |
| ---------- | ---------- |
| `getProviderAuthStatus` | `(provider: string) => AuthStatus` |

#### :gear: getProviderDisplayName

Get display name for a provider.

| Method | Type |
| ---------- | ---------- |
| `getProviderDisplayName` | `(provider: string) => string` |

#### :gear: isUsingOAuth

Check if a model is using OAuth credentials (subscription).

| Method | Type |
| ---------- | ---------- |
| `isUsingOAuth` | `(model: Model<Api>) => boolean` |

#### :gear: registerProvider

Register a provider dynamically (from extensions).

If provider has models: replaces all existing models for this provider.
If provider has only baseUrl/headers: overrides existing models' URLs.
If provider has oauth: registers OAuth provider for /login support.

| Method | Type |
| ---------- | ---------- |
| `registerProvider` | `(providerName: string, config: ProviderConfigInput) => void` |

#### :gear: unregisterProvider

Unregister a previously registered provider.

Removes the provider from the registry and reloads models from disk so that
built-in models overridden by this provider are restored to their original state.
Also resets dynamic OAuth and API stream registrations before reapplying
remaining dynamic providers.
Has no effect if the provider was never registered.

| Method | Type |
| ---------- | ---------- |
| `unregisterProvider` | `(providerName: string) => void` |

## :factory: DynamicBorder

Dynamic border component that adjusts to viewport width.

Note: When used from extensions loaded via jiti, the global `theme` may be undefined
because jiti creates a separate module cache. Always pass an explicit color
function when using DynamicBorder in components exported for extension use.

### Methods

- [color](#gear-color)
- [invalidate](#gear-invalidate)
- [render](#gear-render)

#### :gear: color

| Method | Type |
| ---------- | ---------- |
| `color` | `(str: string) => string` |

#### :gear: invalidate

Invalidate any cached rendering state.
Called when theme changes or when component needs to re-render from scratch.

| Method | Type |
| ---------- | ---------- |
| `invalidate` | `() => void` |

#### :gear: render

Render the component to lines for the given viewport width

| Method | Type |
| ---------- | ---------- |
| `render` | `(width: number) => string[]` |

## :factory: SessionSelectorComponent

Component that renders a session selector

### Methods

- [handleInput](#gear-handleinput)
- [getSessionList](#gear-getsessionlist)

#### :gear: handleInput

| Method | Type |
| ---------- | ---------- |
| `handleInput` | `(data: string) => void` |

#### :gear: getSessionList

| Method | Type |
| ---------- | ---------- |
| `getSessionList` | `() => SessionList` |

## :factory: DefaultResourceLoader

### Methods

- [getExtensions](#gear-getextensions)
- [getSkills](#gear-getskills)
- [getPrompts](#gear-getprompts)
- [getThemes](#gear-getthemes)
- [getAgentsFiles](#gear-getagentsfiles)
- [getSystemPrompt](#gear-getsystemprompt)
- [getAppendSystemPrompt](#gear-getappendsystemprompt)
- [extendResources](#gear-extendresources)

#### :gear: getExtensions

| Method | Type |
| ---------- | ---------- |
| `getExtensions` | `() => LoadExtensionsResult` |

#### :gear: getSkills

| Method | Type |
| ---------- | ---------- |
| `getSkills` | `() => { skills: Skill[]; diagnostics: ResourceDiagnostic[]; }` |

#### :gear: getPrompts

| Method | Type |
| ---------- | ---------- |
| `getPrompts` | `() => { prompts: PromptTemplate[]; diagnostics: ResourceDiagnostic[]; }` |

#### :gear: getThemes

| Method | Type |
| ---------- | ---------- |
| `getThemes` | `() => { themes: Theme[]; diagnostics: ResourceDiagnostic[]; }` |

#### :gear: getAgentsFiles

| Method | Type |
| ---------- | ---------- |
| `getAgentsFiles` | `() => { agentsFiles: { path: string; content: string; }[]; }` |

#### :gear: getSystemPrompt

| Method | Type |
| ---------- | ---------- |
| `getSystemPrompt` | `() => string or undefined` |

#### :gear: getAppendSystemPrompt

| Method | Type |
| ---------- | ---------- |
| `getAppendSystemPrompt` | `() => string[]` |

#### :gear: extendResources

| Method | Type |
| ---------- | ---------- |
| `extendResources` | `(paths: ResourceExtensionPaths) => void` |

## :factory: MissingSessionCwdError

## :factory: ArminComponent

### Methods

- [invalidate](#gear-invalidate)
- [render](#gear-render)
- [dispose](#gear-dispose)

#### :gear: invalidate

Invalidate any cached rendering state.
Called when theme changes or when component needs to re-render from scratch.

| Method | Type |
| ---------- | ---------- |
| `invalidate` | `() => void` |

#### :gear: render

Render the component to lines for the given viewport width

| Method | Type |
| ---------- | ---------- |
| `render` | `(width: number) => string[]` |

#### :gear: dispose

| Method | Type |
| ---------- | ---------- |
| `dispose` | `() => void` |

## :factory: AssistantMessageComponent

Component that renders a complete assistant message

### Methods

- [setHideThinkingBlock](#gear-sethidethinkingblock)
- [setHiddenThinkingLabel](#gear-sethiddenthinkinglabel)
- [updateContent](#gear-updatecontent)

#### :gear: setHideThinkingBlock

| Method | Type |
| ---------- | ---------- |
| `setHideThinkingBlock` | `(hide: boolean) => void` |

#### :gear: setHiddenThinkingLabel

| Method | Type |
| ---------- | ---------- |
| `setHiddenThinkingLabel` | `(label: string) => void` |

#### :gear: updateContent

| Method | Type |
| ---------- | ---------- |
| `updateContent` | `(message: AssistantMessage) => void` |

## :factory: BashExecutionComponent

### Methods

- [borderColor](#gear-bordercolor)
- [setExpanded](#gear-setexpanded)
- [appendOutput](#gear-appendoutput)
- [setComplete](#gear-setcomplete)
- [getOutput](#gear-getoutput)
- [getCommand](#gear-getcommand)

#### :gear: borderColor

| Method | Type |
| ---------- | ---------- |
| `borderColor` | `(str: string) => string` |

#### :gear: setExpanded

Set whether the output is expanded (shows full output) or collapsed (preview only).

| Method | Type |
| ---------- | ---------- |
| `setExpanded` | `(expanded: boolean) => void` |

#### :gear: appendOutput

| Method | Type |
| ---------- | ---------- |
| `appendOutput` | `(chunk: string) => void` |

#### :gear: setComplete

| Method | Type |
| ---------- | ---------- |
| `setComplete` | `(exitCode: number or undefined, cancelled: boolean, truncationResult?: TruncationResult or undefined, fullOutputPath?: string or undefined) => void` |

#### :gear: getOutput

Get the raw output for creating BashExecutionMessage.

| Method | Type |
| ---------- | ---------- |
| `getOutput` | `() => string` |

#### :gear: getCommand

Get the command that was executed.

| Method | Type |
| ---------- | ---------- |
| `getCommand` | `() => string` |

## :factory: BorderedLoader

Loader wrapped with borders for extension UI

### Methods

- [borderColor](#gear-bordercolor)
- [handleInput](#gear-handleinput)
- [dispose](#gear-dispose)

#### :gear: borderColor

| Method | Type |
| ---------- | ---------- |
| `borderColor` | `(s: string) => string` |

#### :gear: handleInput

| Method | Type |
| ---------- | ---------- |
| `handleInput` | `(data: string) => void` |

#### :gear: dispose

| Method | Type |
| ---------- | ---------- |
| `dispose` | `() => void` |

## :factory: BranchSummaryMessageComponent

Component that renders a branch summary message with collapsed/expanded state.
Uses same background color as custom messages for visual consistency.

### Methods

- [setExpanded](#gear-setexpanded)

#### :gear: setExpanded

| Method | Type |
| ---------- | ---------- |
| `setExpanded` | `(expanded: boolean) => void` |

## :factory: CompactionSummaryMessageComponent

Component that renders a compaction message with collapsed/expanded state.
Uses same background color as custom messages for visual consistency.

### Methods

- [setExpanded](#gear-setexpanded)

#### :gear: setExpanded

| Method | Type |
| ---------- | ---------- |
| `setExpanded` | `(expanded: boolean) => void` |

## :factory: CountdownTimer

### Methods

- [dispose](#gear-dispose)

#### :gear: dispose

| Method | Type |
| ---------- | ---------- |
| `dispose` | `() => void` |

## :factory: CustomEditor

Custom editor that handles app-level keybindings for coding-agent.

### Methods

- [onAction](#gear-onaction)
- [handleInput](#gear-handleinput)

#### :gear: onAction

Register a handler for an app action.

| Method | Type |
| ---------- | ---------- |
| `onAction` | `(action: keyof AppKeybindings, handler: () => void) => void` |

#### :gear: handleInput

Optional handler for keyboard input when component has focus

| Method | Type |
| ---------- | ---------- |
| `handleInput` | `(data: string) => void` |

### Properties

- [actionHandlers](#gear-actionhandlers)
- [onEscape](#gear-onescape)
- [onCtrlD](#gear-onctrld)
- [onPasteImage](#gear-onpasteimage)
- [onExtensionShortcut](#gear-onextensionshortcut)

#### :gear: actionHandlers

| Property | Type |
| ---------- | ---------- |
| `actionHandlers` | `Map<keyof AppKeybindings, () => void>` |

#### :gear: onEscape

| Property | Type |
| ---------- | ---------- |
| `onEscape` | `(() => void) or undefined` |

#### :gear: onCtrlD

| Property | Type |
| ---------- | ---------- |
| `onCtrlD` | `(() => void) or undefined` |

#### :gear: onPasteImage

| Property | Type |
| ---------- | ---------- |
| `onPasteImage` | `(() => void) or undefined` |

#### :gear: onExtensionShortcut

Handler for extension-registered shortcuts. Returns true if handled.

| Property | Type |
| ---------- | ---------- |
| `onExtensionShortcut` | `((data: string) => boolean) or undefined` |

## :factory: CustomMessageComponent

Component that renders a custom message entry from extensions.
Uses distinct styling to differentiate from user messages.

### Methods

- [setExpanded](#gear-setexpanded)

#### :gear: setExpanded

| Method | Type |
| ---------- | ---------- |
| `setExpanded` | `(expanded: boolean) => void` |

## :factory: DaxnutsComponent

### Methods

- [invalidate](#gear-invalidate)
- [render](#gear-render)
- [dispose](#gear-dispose)

#### :gear: invalidate

Invalidate any cached rendering state.
Called when theme changes or when component needs to re-render from scratch.

| Method | Type |
| ---------- | ---------- |
| `invalidate` | `() => void` |

#### :gear: render

Render the component to lines for the given viewport width

| Method | Type |
| ---------- | ---------- |
| `render` | `(width: number) => string[]` |

#### :gear: dispose

| Method | Type |
| ---------- | ---------- |
| `dispose` | `() => void` |

## :factory: EarendilAnnouncementComponent

## :factory: ExtensionEditorComponent

### Methods

- [handleInput](#gear-handleinput)

#### :gear: handleInput

| Method | Type |
| ---------- | ---------- |
| `handleInput` | `(keyData: string) => void` |

## :factory: ExtensionInputComponent

### Methods

- [handleInput](#gear-handleinput)
- [dispose](#gear-dispose)

#### :gear: handleInput

| Method | Type |
| ---------- | ---------- |
| `handleInput` | `(keyData: string) => void` |

#### :gear: dispose

| Method | Type |
| ---------- | ---------- |
| `dispose` | `() => void` |

## :factory: ExtensionSelectorComponent

### Methods

- [handleInput](#gear-handleinput)
- [dispose](#gear-dispose)

#### :gear: handleInput

| Method | Type |
| ---------- | ---------- |
| `handleInput` | `(keyData: string) => void` |

#### :gear: dispose

| Method | Type |
| ---------- | ---------- |
| `dispose` | `() => void` |

## :factory: FooterComponent

Footer component that shows pwd, token stats, and context usage.
Computes token/context stats from session, gets git branch and extension statuses from provider.

### Methods

- [setSession](#gear-setsession)
- [setAutoCompactEnabled](#gear-setautocompactenabled)
- [invalidate](#gear-invalidate)
- [dispose](#gear-dispose)
- [render](#gear-render)

#### :gear: setSession

| Method | Type |
| ---------- | ---------- |
| `setSession` | `(session: AgentSession) => void` |

#### :gear: setAutoCompactEnabled

| Method | Type |
| ---------- | ---------- |
| `setAutoCompactEnabled` | `(enabled: boolean) => void` |

#### :gear: invalidate

No-op: git branch caching now handled by provider.
Kept for compatibility with existing call sites in interactive-mode.

| Method | Type |
| ---------- | ---------- |
| `invalidate` | `() => void` |

#### :gear: dispose

Clean up resources.
Git watcher cleanup now handled by provider.

| Method | Type |
| ---------- | ---------- |
| `dispose` | `() => void` |

#### :gear: render

Render the component to lines for the given viewport width

| Method | Type |
| ---------- | ---------- |
| `render` | `(width: number) => string[]` |

## :factory: LoginDialogComponent

Login dialog component - replaces editor during OAuth login flow

### Methods

- [showAuth](#gear-showauth)
- [showManualInput](#gear-showmanualinput)
- [showPrompt](#gear-showprompt)
- [showInfo](#gear-showinfo)
- [showWaiting](#gear-showwaiting)
- [showProgress](#gear-showprogress)
- [handleInput](#gear-handleinput)

#### :gear: showAuth

Called by onAuth callback - show URL and optional instructions

| Method | Type |
| ---------- | ---------- |
| `showAuth` | `(url: string, instructions?: string or undefined) => void` |

#### :gear: showManualInput

Show input for manual code/URL entry (for callback server providers)

| Method | Type |
| ---------- | ---------- |
| `showManualInput` | `(prompt: string) => Promise<string>` |

#### :gear: showPrompt

Called by onPrompt callback - show prompt and wait for input
Note: Does NOT clear content, appends to existing (preserves URL from showAuth)

| Method | Type |
| ---------- | ---------- |
| `showPrompt` | `(message: string, placeholder?: string or undefined) => Promise<string>` |

#### :gear: showInfo

Show informational text without prompting for input.

| Method | Type |
| ---------- | ---------- |
| `showInfo` | `(lines: string[]) => void` |

#### :gear: showWaiting

Show waiting message (for polling flows like GitHub Copilot)

| Method | Type |
| ---------- | ---------- |
| `showWaiting` | `(message: string) => void` |

#### :gear: showProgress

Called by onProgress callback

| Method | Type |
| ---------- | ---------- |
| `showProgress` | `(message: string) => void` |

#### :gear: handleInput

| Method | Type |
| ---------- | ---------- |
| `handleInput` | `(data: string) => void` |

## :factory: ModelSelectorComponent

Component that renders a model selector with search

### Methods

- [handleInput](#gear-handleinput)
- [getSearchInput](#gear-getsearchinput)

#### :gear: handleInput

| Method | Type |
| ---------- | ---------- |
| `handleInput` | `(keyData: string) => void` |

#### :gear: getSearchInput

| Method | Type |
| ---------- | ---------- |
| `getSearchInput` | `() => Input` |

## :factory: OAuthSelectorComponent

Component that renders an auth provider selector

### Methods

- [handleInput](#gear-handleinput)

#### :gear: handleInput

| Method | Type |
| ---------- | ---------- |
| `handleInput` | `(keyData: string) => void` |

## :factory: ScopedModelsSelectorComponent

Component for enabling/disabling models for Ctrl+P cycling.
Changes are session-only until explicitly persisted with Ctrl+S.

### Methods

- [handleInput](#gear-handleinput)
- [getSearchInput](#gear-getsearchinput)

#### :gear: handleInput

| Method | Type |
| ---------- | ---------- |
| `handleInput` | `(data: string) => void` |

#### :gear: getSearchInput

| Method | Type |
| ---------- | ---------- |
| `getSearchInput` | `() => Input` |

## :factory: SettingsSelectorComponent

Main settings selector component.

### Methods

- [items.submenu](#gear-items.submenu)
- [getSettingsList](#gear-getsettingslist)

#### :gear: items.submenu

| Method | Type |
| ---------- | ---------- |
| `items.submenu` | `(_currentValue: string, done: (selectedValue?: string or undefined) => void) => WarningSettingsSubmenu` |

#### :gear: getSettingsList

| Method | Type |
| ---------- | ---------- |
| `getSettingsList` | `() => SettingsList` |

## :factory: SkillInvocationMessageComponent

Component that renders a skill invocation message with collapsed/expanded state.
Uses same background color as custom messages for visual consistency.
Only renders the skill block itself - user message is rendered separately.

### Methods

- [setExpanded](#gear-setexpanded)

#### :gear: setExpanded

| Method | Type |
| ---------- | ---------- |
| `setExpanded` | `(expanded: boolean) => void` |

## :factory: ToolExecutionComponent

### Methods

- [updateArgs](#gear-updateargs)
- [markExecutionStarted](#gear-markexecutionstarted)
- [setArgsComplete](#gear-setargscomplete)
- [updateResult](#gear-updateresult)
- [setExpanded](#gear-setexpanded)
- [setShowImages](#gear-setshowimages)
- [setImageWidthCells](#gear-setimagewidthcells)

#### :gear: updateArgs

| Method | Type |
| ---------- | ---------- |
| `updateArgs` | `(args: any) => void` |

#### :gear: markExecutionStarted

| Method | Type |
| ---------- | ---------- |
| `markExecutionStarted` | `() => void` |

#### :gear: setArgsComplete

| Method | Type |
| ---------- | ---------- |
| `setArgsComplete` | `() => void` |

#### :gear: updateResult

| Method | Type |
| ---------- | ---------- |
| `updateResult` | `(result: { content: { type: string; text?: string or undefined; data?: string or undefined; mimeType?: string or undefined; }[]; details?: any; isError: boolean; }, isPartial?: boolean) => void` |

#### :gear: setExpanded

| Method | Type |
| ---------- | ---------- |
| `setExpanded` | `(expanded: boolean) => void` |

#### :gear: setShowImages

| Method | Type |
| ---------- | ---------- |
| `setShowImages` | `(show: boolean) => void` |

#### :gear: setImageWidthCells

| Method | Type |
| ---------- | ---------- |
| `setImageWidthCells` | `(width: number) => void` |

## :factory: TreeSelectorComponent

Component that renders a session tree selector for navigation

### Methods

- [handleInput](#gear-handleinput)
- [getTreeList](#gear-gettreelist)

#### :gear: handleInput

| Method | Type |
| ---------- | ---------- |
| `handleInput` | `(keyData: string) => void` |

#### :gear: getTreeList

| Method | Type |
| ---------- | ---------- |
| `getTreeList` | `() => TreeList` |

## :factory: UserMessageComponent

Component that renders a user message

## :factory: UserMessageSelectorComponent

Component that renders a user message selector for branching

### Methods

- [getMessageList](#gear-getmessagelist)

#### :gear: getMessageList

| Method | Type |
| ---------- | ---------- |
| `getMessageList` | `() => UserMessageList` |

## :factory: InteractiveMode

### Constructors

`public`: 初始化交互模式的 UI 组件和状态。
创建 TUI 引擎、布局容器（header/chat/pending/status/editor/footer）、
编辑器组件、自动补全、底部状态栏等核心 UI 结构。
此时尚未启动 TUI 渲染——调用 init() 完成启动。

Parameters:

* `runtimeHost`: - Agent 会话运行时的宿主，提供 session 切换和生命周期管理
* `options`: - 启动选项（迁移提示、初始消息等）


### Methods

- [renderInitialMessages](#gear-renderinitialmessages)
- [clearEditor](#gear-cleareditor)
- [showError](#gear-showerror)
- [showWarning](#gear-showwarning)
- [showNewVersionNotification](#gear-shownewversionnotification)
- [showPackageUpdateNotification](#gear-showpackageupdatenotification)
- [stop](#gear-stop)

#### :gear: renderInitialMessages

渲染初始消息——启动时或会话恢复时调用。
从 SessionManager 构建会话上下文并渲染所有历史消息到聊天容器。
同时显示压缩次数提示（如有）。

| Method | Type |
| ---------- | ---------- |
| `renderInitialMessages` | `() => void` |

#### :gear: clearEditor

| Method | Type |
| ---------- | ---------- |
| `clearEditor` | `() => void` |

#### :gear: showError

| Method | Type |
| ---------- | ---------- |
| `showError` | `(errorMessage: string) => void` |

#### :gear: showWarning

| Method | Type |
| ---------- | ---------- |
| `showWarning` | `(warningMessage: string) => void` |

#### :gear: showNewVersionNotification

| Method | Type |
| ---------- | ---------- |
| `showNewVersionNotification` | `(newVersion: string) => void` |

#### :gear: showPackageUpdateNotification

| Method | Type |
| ---------- | ---------- |
| `showPackageUpdateNotification` | `(packages: string[]) => void` |

#### :gear: stop

| Method | Type |
| ---------- | ---------- |
| `stop` | `() => void` |

## :factory: RpcClient

### Methods

- [onEvent](#gear-onevent)
- [getStderr](#gear-getstderr)
- [waitForIdle](#gear-waitforidle)
- [collectEvents](#gear-collectevents)

#### :gear: onEvent

Subscribe to agent events.

| Method | Type |
| ---------- | ---------- |
| `onEvent` | `(listener: RpcEventListener) => () => void` |

#### :gear: getStderr

Get collected stderr output (useful for debugging).

| Method | Type |
| ---------- | ---------- |
| `getStderr` | `() => string` |

#### :gear: waitForIdle

Wait for agent to become idle (no streaming).
Resolves when agent_end event is received.

| Method | Type |
| ---------- | ---------- |
| `waitForIdle` | `(timeout?: number) => Promise<void>` |

#### :gear: collectEvents

Collect events until agent becomes idle.

| Method | Type |
| ---------- | ---------- |
| `collectEvents` | `(timeout?: number) => Promise<AgentEvent[]>` |

## :factory: ShowImagesSelectorComponent

Component that renders a show images selector with borders

### Methods

- [getSelectList](#gear-getselectlist)

#### :gear: getSelectList

| Method | Type |
| ---------- | ---------- |
| `getSelectList` | `() => SelectList` |

## :factory: ThemeSelectorComponent

Component that renders a theme selector

### Methods

- [getSelectList](#gear-getselectlist)

#### :gear: getSelectList

| Method | Type |
| ---------- | ---------- |
| `getSelectList` | `() => SelectList` |

## :factory: ThinkingSelectorComponent

Component that renders a thinking level selector with borders

### Methods

- [getSelectList](#gear-getselectlist)

#### :gear: getSelectList

| Method | Type |
| ---------- | ---------- |
| `getSelectList` | `() => SelectList` |

## :factory: ExtensionRunner

### Methods

- [bindCore](#gear-bindcore)
- [bindCommandContext](#gear-bindcommandcontext)
- [setUIContext](#gear-setuicontext)
- [getUIContext](#gear-getuicontext)
- [hasUI](#gear-hasui)
- [getExtensionPaths](#gear-getextensionpaths)
- [getAllRegisteredTools](#gear-getallregisteredtools)
- [getToolDefinition](#gear-gettooldefinition)
- [getFlags](#gear-getflags)
- [setFlagValue](#gear-setflagvalue)
- [getFlagValues](#gear-getflagvalues)
- [getShortcuts](#gear-getshortcuts)
- [getShortcutDiagnostics](#gear-getshortcutdiagnostics)
- [invalidate](#gear-invalidate)
- [onError](#gear-onerror)
- [emitError](#gear-emiterror)
- [hasHandlers](#gear-hashandlers)
- [getMessageRenderer](#gear-getmessagerenderer)
- [getRegisteredCommands](#gear-getregisteredcommands)
- [getCommandDiagnostics](#gear-getcommanddiagnostics)
- [getCommand](#gear-getcommand)
- [shutdown](#gear-shutdown)
- [createContext](#gear-createcontext)
- [createCommandContext](#gear-createcommandcontext)

#### :gear: bindCore

| Method | Type |
| ---------- | ---------- |
| `bindCore` | `(actions: ExtensionActions, contextActions: ExtensionContextActions, providerActions?: { registerProvider?: ((name: string, config: ProviderConfig) => void) or undefined; unregisterProvider?: ((name: string) => void) or undefined; } or undefined) => void` |

#### :gear: bindCommandContext

| Method | Type |
| ---------- | ---------- |
| `bindCommandContext` | `(actions?: ExtensionCommandContextActions or undefined) => void` |

#### :gear: setUIContext

| Method | Type |
| ---------- | ---------- |
| `setUIContext` | `(uiContext?: ExtensionUIContext or undefined) => void` |

#### :gear: getUIContext

| Method | Type |
| ---------- | ---------- |
| `getUIContext` | `() => ExtensionUIContext` |

#### :gear: hasUI

| Method | Type |
| ---------- | ---------- |
| `hasUI` | `() => boolean` |

#### :gear: getExtensionPaths

| Method | Type |
| ---------- | ---------- |
| `getExtensionPaths` | `() => string[]` |

#### :gear: getAllRegisteredTools

Get all registered tools from all extensions (first registration per name wins).

| Method | Type |
| ---------- | ---------- |
| `getAllRegisteredTools` | `() => RegisteredTool[]` |

#### :gear: getToolDefinition

Get a tool definition by name. Returns undefined if not found.

| Method | Type |
| ---------- | ---------- |
| `getToolDefinition` | `(toolName: string) => ToolDefinition<TSchema, unknown, any> or undefined` |

#### :gear: getFlags

| Method | Type |
| ---------- | ---------- |
| `getFlags` | `() => Map<string, ExtensionFlag>` |

#### :gear: setFlagValue

| Method | Type |
| ---------- | ---------- |
| `setFlagValue` | `(name: string, value: string or boolean) => void` |

#### :gear: getFlagValues

| Method | Type |
| ---------- | ---------- |
| `getFlagValues` | `() => Map<string, string or boolean>` |

#### :gear: getShortcuts

| Method | Type |
| ---------- | ---------- |
| `getShortcuts` | `(resolvedKeybindings: KeybindingsConfig) => Map<KeyId, ExtensionShortcut>` |

#### :gear: getShortcutDiagnostics

| Method | Type |
| ---------- | ---------- |
| `getShortcutDiagnostics` | `() => ResourceDiagnostic[]` |

#### :gear: invalidate

| Method | Type |
| ---------- | ---------- |
| `invalidate` | `(message?: string) => void` |

#### :gear: onError

| Method | Type |
| ---------- | ---------- |
| `onError` | `(listener: ExtensionErrorListener) => () => void` |

#### :gear: emitError

| Method | Type |
| ---------- | ---------- |
| `emitError` | `(error: ExtensionError) => void` |

#### :gear: hasHandlers

| Method | Type |
| ---------- | ---------- |
| `hasHandlers` | `(eventType: string) => boolean` |

#### :gear: getMessageRenderer

| Method | Type |
| ---------- | ---------- |
| `getMessageRenderer` | `(customType: string) => MessageRenderer or undefined` |

#### :gear: getRegisteredCommands

| Method | Type |
| ---------- | ---------- |
| `getRegisteredCommands` | `() => ResolvedCommand[]` |

#### :gear: getCommandDiagnostics

| Method | Type |
| ---------- | ---------- |
| `getCommandDiagnostics` | `() => ResourceDiagnostic[]` |

#### :gear: getCommand

| Method | Type |
| ---------- | ---------- |
| `getCommand` | `(name: string) => ResolvedCommand or undefined` |

#### :gear: shutdown

Request a graceful shutdown. Called by extension tools and event handlers.
The actual shutdown behavior is provided by the mode via bindExtensions().

| Method | Type |
| ---------- | ---------- |
| `shutdown` | `() => void` |

#### :gear: createContext

Create an ExtensionContext for use in event handlers and tool execution.
Context values are resolved at call time, so changes via bindCore/bindUI are reflected.

| Method | Type |
| ---------- | ---------- |
| `createContext` | `() => ExtensionContext` |

#### :gear: createCommandContext

| Method | Type |
| ---------- | ---------- |
| `createCommandContext` | `() => ExtensionCommandContext` |

## :factory: AgentSession

### Methods

- [subscribe](#gear-subscribe)
- [dispose](#gear-dispose)
- [getActiveToolNames](#gear-getactivetoolnames)
- [getAllTools](#gear-getalltools)
- [getToolDefinition](#gear-gettooldefinition)
- [setActiveToolsByName](#gear-setactivetoolsbyname)
- [setScopedModels](#gear-setscopedmodels)
- [clearQueue](#gear-clearqueue)
- [getSteeringMessages](#gear-getsteeringmessages)
- [getFollowUpMessages](#gear-getfollowupmessages)
- [setThinkingLevel](#gear-setthinkinglevel)
- [cycleThinkingLevel](#gear-cyclethinkinglevel)
- [getAvailableThinkingLevels](#gear-getavailablethinkinglevels)
- [supportsThinking](#gear-supportsthinking)
- [setSteeringMode](#gear-setsteeringmode)
- [setFollowUpMode](#gear-setfollowupmode)
- [abortCompaction](#gear-abortcompaction)
- [abortBranchSummary](#gear-abortbranchsummary)
- [setAutoCompactionEnabled](#gear-setautocompactionenabled)
- [abortRetry](#gear-abortretry)
- [setAutoRetryEnabled](#gear-setautoretryenabled)
- [recordBashResult](#gear-recordbashresult)
- [abortBash](#gear-abortbash)
- [setSessionName](#gear-setsessionname)
- [getUserMessagesForForking](#gear-getusermessagesforforking)
- [getSessionStats](#gear-getsessionstats)
- [getContextUsage](#gear-getcontextusage)
- [exportToJsonl](#gear-exporttojsonl)
- [getLastAssistantText](#gear-getlastassistanttext)
- [createReplacedSessionContext](#gear-createreplacedsessioncontext)
- [hasExtensionHandlers](#gear-hasextensionhandlers)

#### :gear: subscribe

订阅会话事件（队列更新、压缩进度、自动重试等）

| Method | Type |
| ---------- | ---------- |
| `subscribe` | `(listener: AgentSessionEventListener) => () => void` |

#### :gear: dispose

释放资源、断开与 Agent 的连接、使扩展运行时失效

| Method | Type |
| ---------- | ---------- |
| `dispose` | `() => void` |

#### :gear: getActiveToolNames

获取当前激活的工具名列表

| Method | Type |
| ---------- | ---------- |
| `getActiveToolNames` | `() => string[]` |

#### :gear: getAllTools

获取所有工具信息（名称、描述、参数 schema、来源）

| Method | Type |
| ---------- | ---------- |
| `getAllTools` | `() => ToolInfo[]` |

#### :gear: getToolDefinition

获取指定名称的工具定义

| Method | Type |
| ---------- | ---------- |
| `getToolDefinition` | `(name: string) => ToolDefinition<TSchema, unknown, any> or undefined` |

#### :gear: setActiveToolsByName

设置激活的工具列表（触发系统提示重建和扩展通知）

| Method | Type |
| ---------- | ---------- |
| `setActiveToolsByName` | `(toolNames: string[]) => void` |

#### :gear: setScopedModels

设置 Ctrl+P 循环切换的模型列表

| Method | Type |
| ---------- | ---------- |
| `setScopedModels` | `(scopedModels: { model: Model<any>; thinkingLevel?: ThinkingLevel or undefined; }[]) => void` |

#### :gear: clearQueue

清除所有队列中的转向和后续消息

| Method | Type |
| ---------- | ---------- |
| `clearQueue` | `() => { steering: string[]; followUp: string[]; }` |

#### :gear: getSteeringMessages

获取转向队列中的消息列表

| Method | Type |
| ---------- | ---------- |
| `getSteeringMessages` | `() => readonly string[]` |

#### :gear: getFollowUpMessages

获取后续队列中的消息列表

| Method | Type |
| ---------- | ---------- |
| `getFollowUpMessages` | `() => readonly string[]` |

#### :gear: setThinkingLevel

设置思考级别（根据模型能力限制范围）

| Method | Type |
| ---------- | ---------- |
| `setThinkingLevel` | `(level: ThinkingLevel) => void` |

#### :gear: cycleThinkingLevel

在当前模型支持的思考级别列表中循环切换

| Method | Type |
| ---------- | ---------- |
| `cycleThinkingLevel` | `() => ThinkingLevel or undefined` |

#### :gear: getAvailableThinkingLevels

Get available thinking levels for current model.
The provider will clamp to what the specific model supports internally.

| Method | Type |
| ---------- | ---------- |
| `getAvailableThinkingLevels` | `() => ThinkingLevel[]` |

#### :gear: supportsThinking

Check if current model supports thinking/reasoning.

| Method | Type |
| ---------- | ---------- |
| `supportsThinking` | `() => boolean` |

#### :gear: setSteeringMode

Set steering message mode.
Saves to settings.

| Method | Type |
| ---------- | ---------- |
| `setSteeringMode` | `(mode: "all" or "one-at-a-time") => void` |

#### :gear: setFollowUpMode

Set follow-up message mode.
Saves to settings.

| Method | Type |
| ---------- | ---------- |
| `setFollowUpMode` | `(mode: "all" or "one-at-a-time") => void` |

#### :gear: abortCompaction

Cancel in-progress compaction (manual or auto).

| Method | Type |
| ---------- | ---------- |
| `abortCompaction` | `() => void` |

#### :gear: abortBranchSummary

Cancel in-progress branch summarization.

| Method | Type |
| ---------- | ---------- |
| `abortBranchSummary` | `() => void` |

#### :gear: setAutoCompactionEnabled

Toggle auto-compaction setting.

| Method | Type |
| ---------- | ---------- |
| `setAutoCompactionEnabled` | `(enabled: boolean) => void` |

#### :gear: abortRetry

Cancel in-progress retry.

| Method | Type |
| ---------- | ---------- |
| `abortRetry` | `() => void` |

#### :gear: setAutoRetryEnabled

Toggle auto-retry setting.

| Method | Type |
| ---------- | ---------- |
| `setAutoRetryEnabled` | `(enabled: boolean) => void` |

#### :gear: recordBashResult

Record a bash execution result in session history.
Used by executeBash and by extensions that handle bash execution themselves.

| Method | Type |
| ---------- | ---------- |
| `recordBashResult` | `(command: string, result: BashResult, options?: { excludeFromContext?: boolean or undefined; } or undefined) => void` |

#### :gear: abortBash

Cancel running bash command.

| Method | Type |
| ---------- | ---------- |
| `abortBash` | `() => void` |

#### :gear: setSessionName

Set a display name for the current session.

| Method | Type |
| ---------- | ---------- |
| `setSessionName` | `(name: string) => void` |

#### :gear: getUserMessagesForForking

Get all user messages from session for fork selector.

| Method | Type |
| ---------- | ---------- |
| `getUserMessagesForForking` | `() => { entryId: string; text: string; }[]` |

#### :gear: getSessionStats

Get session statistics.

| Method | Type |
| ---------- | ---------- |
| `getSessionStats` | `() => SessionStats` |

#### :gear: getContextUsage

| Method | Type |
| ---------- | ---------- |
| `getContextUsage` | `() => ContextUsage or undefined` |

#### :gear: exportToJsonl

Export the current session branch to a JSONL file.
Writes the session header followed by all entries on the current branch path.

| Method | Type |
| ---------- | ---------- |
| `exportToJsonl` | `(outputPath?: string or undefined) => string` |

Parameters:

* `outputPath`: Target file path. If omitted, generates a timestamped file in cwd.


Returns:

The resolved output file path.

#### :gear: getLastAssistantText

Get text content of last assistant message.
Useful for /copy command.

| Method | Type |
| ---------- | ---------- |
| `getLastAssistantText` | `() => string or undefined` |

Returns:

Text content, or undefined if no assistant message exists

#### :gear: createReplacedSessionContext

| Method | Type |
| ---------- | ---------- |
| `createReplacedSessionContext` | `() => ReplacedSessionContext` |

#### :gear: hasExtensionHandlers

Check if extensions have handlers for a specific event type.

| Method | Type |
| ---------- | ---------- |
| `hasExtensionHandlers` | `(eventType: string) => boolean` |

## :factory: SessionImportFileNotFoundError

Thrown when /import references a JSONL file path that does not exist.

## :factory: AgentSessionRuntime

Owns the current AgentSession plus its cwd-bound services.

Session replacement methods tear down the current runtime first, then create
and apply the next runtime. If creation fails, the error is propagated to the
caller. The caller is responsible for user-facing error handling.

### Methods

- [setRebindSession](#gear-setrebindsession)
- [setBeforeSessionInvalidate](#gear-setbeforesessioninvalidate)

#### :gear: setRebindSession

| Method | Type |
| ---------- | ---------- |
| `setRebindSession` | `(rebindSession?: ((session: AgentSession) => Promise<void>) or undefined) => void` |

#### :gear: setBeforeSessionInvalidate

Set a synchronous callback that runs after `session_shutdown` handlers finish
but before the current session is invalidated.

This is for host-owned UI teardown that must not yield to the event loop,
such as detaching extension-provided TUI components before the old extension
context becomes stale.

| Method | Type |
| ---------- | ---------- |
| `setBeforeSessionInvalidate` | `(beforeSessionInvalidate?: (() => void) or undefined) => void` |

## :factory: ConfigSelectorComponent

### Methods

- [getResourceList](#gear-getresourcelist)

#### :gear: getResourceList

| Method | Type |
| ---------- | ---------- |
| `getResourceList` | `() => ResourceList` |

## :tropical_drink: Interfaces

- [SelfUpdateCommand](#gear-selfupdatecommand)
- [CompactionSettings](#gear-compactionsettings)
- [BranchSummarySettings](#gear-branchsummarysettings)
- [ProviderRetrySettings](#gear-providerretrysettings)
- [RetrySettings](#gear-retrysettings)
- [TerminalSettings](#gear-terminalsettings)
- [ImageSettings](#gear-imagesettings)
- [ThinkingBudgetsSettings](#gear-thinkingbudgetssettings)
- [MarkdownSettings](#gear-markdownsettings)
- [WarningSettings](#gear-warningsettings)
- [Settings](#gear-settings)
- [SettingsStorage](#gear-settingsstorage)
- [SettingsError](#gear-settingserror)
- [PathMetadata](#gear-pathmetadata)
- [ResolvedResource](#gear-resolvedresource)
- [ResolvedPaths](#gear-resolvedpaths)
- [ProgressEvent](#gear-progressevent)
- [PackageUpdate](#gear-packageupdate)
- [ConfiguredPackage](#gear-configuredpackage)
- [PackageManager](#gear-packagemanager)
- [SourceInfo](#gear-sourceinfo)
- [ThemeInfo](#gear-themeinfo)
- [ShellConfig](#gear-shellconfig)
- [KeyTextFormatOptions](#gear-keytextformatoptions)
- [VisualTruncateResult](#gear-visualtruncateresult)
- [TruncationResult](#gear-truncationresult)
- [TruncationOptions](#gear-truncationoptions)
- [OutputAccumulatorOptions](#gear-outputaccumulatoroptions)
- [OutputSnapshot](#gear-outputsnapshot)
- [BashToolDetails](#gear-bashtooldetails)
- [BashOperations](#gear-bashoperations)
- [BashSpawnContext](#gear-bashspawncontext)
- [BashToolOptions](#gear-bashtooloptions)
- [BashExecutorOptions](#gear-bashexecutoroptions)
- [BashResult](#gear-bashresult)
- [BashExecutionMessage](#gear-bashexecutionmessage)
- [CustomMessage](#gear-custommessage)
- [BranchSummaryMessage](#gear-branchsummarymessage)
- [CompactionSummaryMessage](#gear-compactionsummarymessage)
- [SessionHeader](#gear-sessionheader)
- [NewSessionOptions](#gear-newsessionoptions)
- [SessionEntryBase](#gear-sessionentrybase)
- [SessionMessageEntry](#gear-sessionmessageentry)
- [ThinkingLevelChangeEntry](#gear-thinkinglevelchangeentry)
- [ModelChangeEntry](#gear-modelchangeentry)
- [CompactionEntry](#gear-compactionentry)
- [BranchSummaryEntry](#gear-branchsummaryentry)
- [CustomEntry](#gear-customentry)
- [LabelEntry](#gear-labelentry)
- [SessionInfoEntry](#gear-sessioninfoentry)
- [CustomMessageEntry](#gear-custommessageentry)
- [SessionTreeNode](#gear-sessiontreenode)
- [SessionContext](#gear-sessioncontext)
- [SessionInfo](#gear-sessioninfo)
- [FileOperations](#gear-fileoperations)
- [CompactionDetails](#gear-compactiondetails)
- [CompactionResult](#gear-compactionresult)
- [CompactionSettings](#gear-compactionsettings)
- [ContextUsageEstimate](#gear-contextusageestimate)
- [CutPointResult](#gear-cutpointresult)
- [CompactionPreparation](#gear-compactionpreparation)
- [BranchSummaryResult](#gear-branchsummaryresult)
- [BranchSummaryDetails](#gear-branchsummarydetails)
- [BranchPreparation](#gear-branchpreparation)
- [CollectEntriesResult](#gear-collectentriesresult)
- [GenerateBranchSummaryOptions](#gear-generatebranchsummaryoptions)
- [EventBus](#gear-eventbus)
- [EventBusController](#gear-eventbuscontroller)
- [ExecOptions](#gear-execoptions)
- [ExecResult](#gear-execresult)
- [AppKeybindings](#gear-appkeybindings)
- [AuthStorageBackend](#gear-authstoragebackend)
- [ProviderConfigInput](#gear-providerconfiginput)
- [SlashCommandInfo](#gear-slashcommandinfo)
- [BuiltinSlashCommand](#gear-builtinslashcommand)
- [ResourceCollision](#gear-resourcecollision)
- [ResourceDiagnostic](#gear-resourcediagnostic)
- [SkillFrontmatter](#gear-skillfrontmatter)
- [Skill](#gear-skill)
- [LoadSkillsResult](#gear-loadskillsresult)
- [LoadSkillsFromDirOptions](#gear-loadskillsfromdiroptions)
- [LoadSkillsOptions](#gear-loadskillsoptions)
- [BuildSystemPromptOptions](#gear-buildsystempromptoptions)
- [RenderDiffOptions](#gear-renderdiffoptions)
- [FuzzyMatchResult](#gear-fuzzymatchresult)
- [Edit](#gear-edit)
- [AppliedEditsResult](#gear-appliededitsresult)
- [EditDiffResult](#gear-editdiffresult)
- [EditDiffError](#gear-editdifferror)
- [EditToolDetails](#gear-edittooldetails)
- [EditOperations](#gear-editoperations)
- [EditToolOptions](#gear-edittooloptions)
- [FindToolDetails](#gear-findtooldetails)
- [FindOperations](#gear-findoperations)
- [FindToolOptions](#gear-findtooloptions)
- [GrepToolDetails](#gear-greptooldetails)
- [GrepOperations](#gear-grepoperations)
- [GrepToolOptions](#gear-greptooloptions)
- [LsToolDetails](#gear-lstooldetails)
- [LsOperations](#gear-lsoperations)
- [LsToolOptions](#gear-lstooloptions)
- [ImageResizeOptions](#gear-imageresizeoptions)
- [ResizedImage](#gear-resizedimage)
- [ReadToolDetails](#gear-readtooldetails)
- [ReadOperations](#gear-readoperations)
- [ReadToolOptions](#gear-readtooloptions)
- [WriteOperations](#gear-writeoperations)
- [WriteToolOptions](#gear-writetooloptions)
- [ToolsOptions](#gear-toolsoptions)
- [ExtensionUIDialogOptions](#gear-extensionuidialogoptions)
- [ExtensionWidgetOptions](#gear-extensionwidgetoptions)
- [WorkingIndicatorOptions](#gear-workingindicatoroptions)
- [ExtensionUIContext](#gear-extensionuicontext)
- [ContextUsage](#gear-contextusage)
- [CompactOptions](#gear-compactoptions)
- [ExtensionContext](#gear-extensioncontext)
- [ExtensionCommandContext](#gear-extensioncommandcontext)
- [ReplacedSessionContext](#gear-replacedsessioncontext)
- [ToolRenderResultOptions](#gear-toolrenderresultoptions)
- [ToolRenderContext](#gear-toolrendercontext)
- [ToolDefinition](#gear-tooldefinition)
- [ResourcesDiscoverEvent](#gear-resourcesdiscoverevent)
- [ResourcesDiscoverResult](#gear-resourcesdiscoverresult)
- [SessionStartEvent](#gear-sessionstartevent)
- [SessionBeforeSwitchEvent](#gear-sessionbeforeswitchevent)
- [SessionBeforeForkEvent](#gear-sessionbeforeforkevent)
- [SessionBeforeCompactEvent](#gear-sessionbeforecompactevent)
- [SessionCompactEvent](#gear-sessioncompactevent)
- [SessionShutdownEvent](#gear-sessionshutdownevent)
- [TreePreparation](#gear-treepreparation)
- [SessionBeforeTreeEvent](#gear-sessionbeforetreeevent)
- [SessionTreeEvent](#gear-sessiontreeevent)
- [ContextEvent](#gear-contextevent)
- [BeforeProviderRequestEvent](#gear-beforeproviderrequestevent)
- [AfterProviderResponseEvent](#gear-afterproviderresponseevent)
- [BeforeAgentStartEvent](#gear-beforeagentstartevent)
- [AgentStartEvent](#gear-agentstartevent)
- [AgentEndEvent](#gear-agentendevent)
- [TurnStartEvent](#gear-turnstartevent)
- [TurnEndEvent](#gear-turnendevent)
- [MessageStartEvent](#gear-messagestartevent)
- [MessageUpdateEvent](#gear-messageupdateevent)
- [MessageEndEvent](#gear-messageendevent)
- [ToolExecutionStartEvent](#gear-toolexecutionstartevent)
- [ToolExecutionUpdateEvent](#gear-toolexecutionupdateevent)
- [ToolExecutionEndEvent](#gear-toolexecutionendevent)
- [ModelSelectEvent](#gear-modelselectevent)
- [ThinkingLevelSelectEvent](#gear-thinkinglevelselectevent)
- [UserBashEvent](#gear-userbashevent)
- [InputEvent](#gear-inputevent)
- [BashToolCallEvent](#gear-bashtoolcallevent)
- [ReadToolCallEvent](#gear-readtoolcallevent)
- [EditToolCallEvent](#gear-edittoolcallevent)
- [WriteToolCallEvent](#gear-writetoolcallevent)
- [GrepToolCallEvent](#gear-greptoolcallevent)
- [FindToolCallEvent](#gear-findtoolcallevent)
- [LsToolCallEvent](#gear-lstoolcallevent)
- [CustomToolCallEvent](#gear-customtoolcallevent)
- [BashToolResultEvent](#gear-bashtoolresultevent)
- [ReadToolResultEvent](#gear-readtoolresultevent)
- [EditToolResultEvent](#gear-edittoolresultevent)
- [WriteToolResultEvent](#gear-writetoolresultevent)
- [GrepToolResultEvent](#gear-greptoolresultevent)
- [FindToolResultEvent](#gear-findtoolresultevent)
- [LsToolResultEvent](#gear-lstoolresultevent)
- [CustomToolResultEvent](#gear-customtoolresultevent)
- [ContextEventResult](#gear-contexteventresult)
- [ToolCallEventResult](#gear-toolcalleventresult)
- [UserBashEventResult](#gear-userbasheventresult)
- [ToolResultEventResult](#gear-toolresulteventresult)
- [MessageEndEventResult](#gear-messageendeventresult)
- [BeforeAgentStartEventResult](#gear-beforeagentstarteventresult)
- [SessionBeforeSwitchResult](#gear-sessionbeforeswitchresult)
- [SessionBeforeForkResult](#gear-sessionbeforeforkresult)
- [SessionBeforeCompactResult](#gear-sessionbeforecompactresult)
- [SessionBeforeTreeResult](#gear-sessionbeforetreeresult)
- [MessageRenderOptions](#gear-messagerenderoptions)
- [RegisteredCommand](#gear-registeredcommand)
- [ResolvedCommand](#gear-resolvedcommand)
- [ExtensionAPI](#gear-extensionapi)
- [ProviderConfig](#gear-providerconfig)
- [ProviderModelConfig](#gear-providermodelconfig)
- [RegisteredTool](#gear-registeredtool)
- [ExtensionFlag](#gear-extensionflag)
- [ExtensionShortcut](#gear-extensionshortcut)
- [ExtensionRuntimeState](#gear-extensionruntimestate)
- [ExtensionActions](#gear-extensionactions)
- [ExtensionContextActions](#gear-extensioncontextactions)
- [ExtensionCommandContextActions](#gear-extensioncommandcontextactions)
- [ExtensionRuntime](#gear-extensionruntime)
- [Extension](#gear-extension)
- [LoadExtensionsResult](#gear-loadextensionsresult)
- [ExtensionError](#gear-extensionerror)
- [Args](#gear-args)
- [ProcessedFiles](#gear-processedfiles)
- [ProcessFileOptions](#gear-processfileoptions)
- [InitialMessageInput](#gear-initialmessageinput)
- [InitialMessageResult](#gear-initialmessageresult)
- [ParsedSearchQuery](#gear-parsedsearchquery)
- [MatchResult](#gear-matchresult)
- [ToolHtmlRenderer](#gear-toolhtmlrenderer)
- [ExportOptions](#gear-exportoptions)
- [ToolHtmlRendererDeps](#gear-toolhtmlrendererdeps)
- [ToolHtmlRenderer](#gear-toolhtmlrenderer)
- [PromptTemplate](#gear-prompttemplate)
- [LoadPromptTemplatesOptions](#gear-loadprompttemplatesoptions)
- [ResourceExtensionPaths](#gear-resourceextensionpaths)
- [ResourceLoader](#gear-resourceloader)
- [DefaultResourceLoaderOptions](#gear-defaultresourceloaderoptions)
- [ScopedModel](#gear-scopedmodel)
- [ParsedModelResult](#gear-parsedmodelresult)
- [ResolveCliModelResult](#gear-resolveclimodelresult)
- [InitialModelResult](#gear-initialmodelresult)
- [CreateAgentSessionOptions](#gear-createagentsessionoptions)
- [CreateAgentSessionResult](#gear-createagentsessionresult)
- [SessionCwdIssue](#gear-sessioncwdissue)
- [ChangelogEntry](#gear-changelogentry)
- [LatestPiRelease](#gear-latestpirelease)
- [ExtensionInputOptions](#gear-extensioninputoptions)
- [ExtensionSelectorOptions](#gear-extensionselectoroptions)
- [ModelsConfig](#gear-modelsconfig)
- [ModelsCallbacks](#gear-modelscallbacks)
- [SettingsConfig](#gear-settingsconfig)
- [SettingsCallbacks](#gear-settingscallbacks)
- [ToolExecutionOptions](#gear-toolexecutionoptions)
- [InteractiveModeOptions](#gear-interactivemodeoptions)
- [PrintModeOptions](#gear-printmodeoptions)
- [RpcSlashCommand](#gear-rpcslashcommand)
- [RpcSessionState](#gear-rpcsessionstate)
- [RpcClientOptions](#gear-rpcclientoptions)
- [ModelInfo](#gear-modelinfo)
- [ParsedSkillBlock](#gear-parsedskillblock)
- [AgentSessionConfig](#gear-agentsessionconfig)
- [ExtensionBindings](#gear-extensionbindings)
- [PromptOptions](#gear-promptoptions)
- [ModelCycleResult](#gear-modelcycleresult)
- [SessionStats](#gear-sessionstats)
- [AgentSessionRuntimeDiagnostic](#gear-agentsessionruntimediagnostic)
- [CreateAgentSessionServicesOptions](#gear-createagentsessionservicesoptions)
- [CreateAgentSessionFromServicesOptions](#gear-createagentsessionfromservicesoptions)
- [AgentSessionServices](#gear-agentsessionservices)
- [CreateAgentSessionRuntimeResult](#gear-createagentsessionruntimeresult)
- [ConfigSelectorOptions](#gear-configselectoroptions)
- [MainOptions](#gear-mainoptions)

### :gear: SelfUpdateCommand

自更新命令。
当包名发生变更时（如从旧包名迁移到新包名），需要先卸载旧包再安装新包，
此时 `steps` 包含多个步骤；否则 `steps` 为空，命令直接执行即可。

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `steps` | `SelfUpdateCommandStep[] or undefined` |  |


### :gear: CompactionSettings



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `enabled` | `boolean or undefined` |  |
| `reserveTokens` | `number or undefined` |  |
| `keepRecentTokens` | `number or undefined` |  |


### :gear: BranchSummarySettings



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `reserveTokens` | `number or undefined` |  |
| `skipPrompt` | `boolean or undefined` |  |


### :gear: ProviderRetrySettings



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `timeoutMs` | `number or undefined` |  |
| `maxRetries` | `number or undefined` |  |
| `maxRetryDelayMs` | `number or undefined` |  |


### :gear: RetrySettings



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `enabled` | `boolean or undefined` |  |
| `maxRetries` | `number or undefined` |  |
| `baseDelayMs` | `number or undefined` |  |
| `provider` | `ProviderRetrySettings or undefined` |  |


### :gear: TerminalSettings



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `showImages` | `boolean or undefined` |  |
| `imageWidthCells` | `number or undefined` |  |
| `clearOnShrink` | `boolean or undefined` |  |
| `showTerminalProgress` | `boolean or undefined` |  |


### :gear: ImageSettings



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `autoResize` | `boolean or undefined` |  |
| `blockImages` | `boolean or undefined` |  |


### :gear: ThinkingBudgetsSettings



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `minimal` | `number or undefined` |  |
| `low` | `number or undefined` |  |
| `medium` | `number or undefined` |  |
| `high` | `number or undefined` |  |


### :gear: MarkdownSettings



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `codeBlockIndent` | `string or undefined` |  |


### :gear: WarningSettings



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `anthropicExtraUsage` | `boolean or undefined` |  |


### :gear: Settings



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `lastChangelogVersion` | `string or undefined` |  |
| `defaultProvider` | `string or undefined` |  |
| `defaultModel` | `string or undefined` |  |
| `defaultThinkingLevel` | `"off" or "minimal" or "low" or "medium" or "high" or "xhigh" or undefined` |  |
| `transport` | `Transport or undefined` |  |
| `steeringMode` | `"all" or "one-at-a-time" or undefined` |  |
| `followUpMode` | `"all" or "one-at-a-time" or undefined` |  |
| `theme` | `string or undefined` |  |
| `compaction` | `CompactionSettings or undefined` |  |
| `branchSummary` | `BranchSummarySettings or undefined` |  |
| `retry` | `RetrySettings or undefined` |  |
| `hideThinkingBlock` | `boolean or undefined` |  |
| `shellPath` | `string or undefined` |  |
| `quietStartup` | `boolean or undefined` |  |
| `shellCommandPrefix` | `string or undefined` |  |
| `npmCommand` | `string[] or undefined` |  |
| `collapseChangelog` | `boolean or undefined` |  |
| `enableInstallTelemetry` | `boolean or undefined` |  |
| `packages` | `PackageSource[] or undefined` |  |
| `extensions` | `string[] or undefined` |  |
| `skills` | `string[] or undefined` |  |
| `prompts` | `string[] or undefined` |  |
| `themes` | `string[] or undefined` |  |
| `enableSkillCommands` | `boolean or undefined` |  |
| `terminal` | `TerminalSettings or undefined` |  |
| `images` | `ImageSettings or undefined` |  |
| `enabledModels` | `string[] or undefined` |  |
| `doubleEscapeAction` | `"fork" or "tree" or "none" or undefined` |  |
| `treeFilterMode` | `"all" or "default" or "no-tools" or "user-only" or "labeled-only" or undefined` |  |
| `thinkingBudgets` | `ThinkingBudgetsSettings or undefined` |  |
| `editorPaddingX` | `number or undefined` |  |
| `autocompleteMaxVisible` | `number or undefined` |  |
| `showHardwareCursor` | `boolean or undefined` |  |
| `markdown` | `MarkdownSettings or undefined` |  |
| `warnings` | `WarningSettings or undefined` |  |
| `sessionDir` | `string or undefined` |  |


### :gear: SettingsStorage



| Property | Type | Description |
| ---------- | ---------- | ---------- |


### :gear: SettingsError



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `scope` | `SettingsScope` |  |
| `error` | `Error` |  |


### :gear: PathMetadata



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `source` | `string` |  |
| `scope` | `SourceScope` |  |
| `origin` | `"package" or "top-level"` |  |
| `baseDir` | `string or undefined` |  |


### :gear: ResolvedResource



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `path` | `string` |  |
| `enabled` | `boolean` |  |
| `metadata` | `PathMetadata` |  |


### :gear: ResolvedPaths



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `extensions` | `ResolvedResource[]` |  |
| `skills` | `ResolvedResource[]` |  |
| `prompts` | `ResolvedResource[]` |  |
| `themes` | `ResolvedResource[]` |  |


### :gear: ProgressEvent



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"start" or "progress" or "complete" or "error"` |  |
| `action` | `"install" or "remove" or "update" or "clone" or "pull"` |  |
| `source` | `string` |  |
| `message` | `string or undefined` |  |


### :gear: PackageUpdate



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `source` | `string` |  |
| `displayName` | `string` |  |
| `type` | `"npm" or "git"` |  |
| `scope` | `"project" or "user"` |  |


### :gear: ConfiguredPackage



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `source` | `string` |  |
| `scope` | `"project" or "user"` |  |
| `filtered` | `boolean` |  |
| `installedPath` | `string or undefined` |  |


### :gear: PackageManager



| Property | Type | Description |
| ---------- | ---------- | ---------- |


### :gear: SourceInfo



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `path` | `string` |  |
| `source` | `string` |  |
| `scope` | `SourceScope` |  |
| `origin` | `SourceOrigin` |  |
| `baseDir` | `string or undefined` |  |


### :gear: ThemeInfo



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `name` | `string` |  |
| `path` | `string or undefined` |  |


### :gear: ShellConfig



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `shell` | `string` |  |
| `args` | `string[]` |  |


### :gear: KeyTextFormatOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `capitalize` | `boolean or undefined` |  |


### :gear: VisualTruncateResult



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `visualLines` | `string[]` | The visual lines to display |
| `skippedCount` | `number` | Number of visual lines that were skipped (hidden) |


### :gear: TruncationResult



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `content` | `string` | The truncated content |
| `truncated` | `boolean` | Whether truncation occurred |
| `truncatedBy` | `"lines" or "bytes" or null` | Which limit was hit: "lines", "bytes", or null if not truncated |
| `totalLines` | `number` | Total number of lines in the original content |
| `totalBytes` | `number` | Total number of bytes in the original content |
| `outputLines` | `number` | Number of complete lines in the truncated output |
| `outputBytes` | `number` | Number of bytes in the truncated output |
| `lastLinePartial` | `boolean` | Whether the last line was partially truncated (only for tail truncation edge case) |
| `firstLineExceedsLimit` | `boolean` | Whether the first line exceeded the byte limit (for head truncation) |
| `maxLines` | `number` | The max lines limit that was applied |
| `maxBytes` | `number` | The max bytes limit that was applied |


### :gear: TruncationOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `maxLines` | `number or undefined` | Maximum number of lines (default: 2000) |
| `maxBytes` | `number or undefined` | Maximum number of bytes (default: 50KB) |


### :gear: OutputAccumulatorOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `maxLines` | `number or undefined` |  |
| `maxBytes` | `number or undefined` |  |
| `tempFilePrefix` | `string or undefined` |  |


### :gear: OutputSnapshot



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `content` | `string` |  |
| `truncation` | `TruncationResult` |  |
| `fullOutputPath` | `string or undefined` |  |


### :gear: BashToolDetails

Bash 工具的执行详情（截断信息和完整输出路径）

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `truncation` | `TruncationResult or undefined` |  |
| `fullOutputPath` | `string or undefined` |  |


### :gear: BashOperations

BashOperations — 可插拔的命令执行后端。
默认实现为本地 shell 执行，可替换为远程执行（如 SSH）。

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `exec` | `(command: string, cwd: string, options: { onData: (data: Buffer<ArrayBufferLike>) => void; signal?: AbortSignal or undefined; timeout?: number or undefined; env?: ProcessEnv or undefined; }) => Promise<...>` | Execute a command and stream output. param: command The command to executeparam: cwd Working directoryparam: options Execution optionsreturns: Promise resolving to exit code (null if killed) |


### :gear: BashSpawnContext

Bash 命令执行的 spawn 上下文

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `command` | `string` |  |
| `cwd` | `string` |  |
| `env` | `ProcessEnv` |  |


### :gear: BashToolOptions

Bash 工具的配置选项

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `operations` | `BashOperations or undefined` | 自定义命令执行后端，默认为本地 shell |
| `commandPrefix` | `string or undefined` | 每条命令前追加的前缀（如 shell 初始化命令） |
| `shellPath` | `string or undefined` | 显式指定的 shell 路径 |
| `spawnHook` | `BashSpawnHook or undefined` | 在执行前调整命令、工作目录或环境变量的钩子 |


### :gear: BashExecutorOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `onChunk` | `((chunk: string) => void) or undefined` | Callback for streaming output chunks (already sanitized) |
| `signal` | `AbortSignal or undefined` | AbortSignal for cancellation |


### :gear: BashResult



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `output` | `string` | Combined stdout + stderr output (sanitized, possibly truncated) |
| `exitCode` | `number or undefined` | Process exit code (undefined if killed/cancelled) |
| `cancelled` | `boolean` | Whether the command was cancelled via signal |
| `truncated` | `boolean` | Whether the output was truncated |
| `fullOutputPath` | `string or undefined` | Path to temp file containing full output (if output exceeded truncation threshold) |


### :gear: BashExecutionMessage

Message type for bash executions via the ! command.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `role` | `"bashExecution"` |  |
| `command` | `string` |  |
| `output` | `string` |  |
| `exitCode` | `number or undefined` |  |
| `cancelled` | `boolean` |  |
| `truncated` | `boolean` |  |
| `fullOutputPath` | `string or undefined` |  |
| `timestamp` | `number` |  |
| `excludeFromContext` | `boolean or undefined` | If true, this message is excluded from LLM context (!! prefix) |


### :gear: CustomMessage

Message type for extension-injected messages via sendMessage().
These are custom messages that extensions can inject into the conversation.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `role` | `"custom"` |  |
| `customType` | `string` |  |
| `content` | `string or (TextContent or ImageContent)[]` |  |
| `display` | `boolean` |  |
| `details` | `T or undefined` |  |
| `timestamp` | `number` |  |


### :gear: BranchSummaryMessage



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `role` | `"branchSummary"` |  |
| `summary` | `string` |  |
| `fromId` | `string` |  |
| `timestamp` | `number` |  |


### :gear: CompactionSummaryMessage



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `role` | `"compactionSummary"` |  |
| `summary` | `string` |  |
| `tokensBefore` | `number` |  |
| `timestamp` | `number` |  |


### :gear: SessionHeader



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"session"` |  |
| `version` | `number or undefined` |  |
| `id` | `string` |  |
| `timestamp` | `string` |  |
| `cwd` | `string` |  |
| `parentSession` | `string or undefined` |  |


### :gear: NewSessionOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `id` | `string or undefined` |  |
| `parentSession` | `string or undefined` |  |


### :gear: SessionEntryBase



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `string` |  |
| `id` | `string` |  |
| `parentId` | `string or null` |  |
| `timestamp` | `string` |  |


### :gear: SessionMessageEntry



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"message"` |  |
| `message` | `AgentMessage` |  |


### :gear: ThinkingLevelChangeEntry



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"thinking_level_change"` |  |
| `thinkingLevel` | `string` |  |


### :gear: ModelChangeEntry



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"model_change"` |  |
| `provider` | `string` |  |
| `modelId` | `string` |  |


### :gear: CompactionEntry



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"compaction"` |  |
| `summary` | `string` |  |
| `firstKeptEntryId` | `string` |  |
| `tokensBefore` | `number` |  |
| `details` | `T or undefined` | Extension-specific data (e.g., ArtifactIndex, version markers for structured compaction) |
| `fromHook` | `boolean or undefined` | True if generated by an extension, undefined/false if pi-generated (backward compatible) |


### :gear: BranchSummaryEntry



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"branch_summary"` |  |
| `fromId` | `string` |  |
| `summary` | `string` |  |
| `details` | `T or undefined` | Extension-specific data (not sent to LLM) |
| `fromHook` | `boolean or undefined` | True if generated by an extension, false if pi-generated |


### :gear: CustomEntry

Custom entry for extensions to store extension-specific data in the session.
Use customType to identify your extension's entries.

Purpose: Persist extension state across session reloads. On reload, extensions can
scan entries for their customType and reconstruct internal state.

Does NOT participate in LLM context (ignored by buildSessionContext).
For injecting content into context, see CustomMessageEntry.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"custom"` |  |
| `customType` | `string` |  |
| `data` | `T or undefined` |  |


### :gear: LabelEntry

Label entry for user-defined bookmarks/markers on entries.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"label"` |  |
| `targetId` | `string` |  |
| `label` | `string or undefined` |  |


### :gear: SessionInfoEntry

Session metadata entry (e.g., user-defined display name).

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"session_info"` |  |
| `name` | `string or undefined` |  |


### :gear: CustomMessageEntry

Custom message entry for extensions to inject messages into LLM context.
Use customType to identify your extension's entries.

Unlike CustomEntry, this DOES participate in LLM context.
The content is converted to a user message in buildSessionContext().
Use details for extension-specific metadata (not sent to LLM).

display controls TUI rendering:
- false: hidden entirely
- true: rendered with distinct styling (different from user messages)

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"custom_message"` |  |
| `customType` | `string` |  |
| `content` | `string or (TextContent or ImageContent)[]` |  |
| `details` | `T or undefined` |  |
| `display` | `boolean` |  |


### :gear: SessionTreeNode

Tree node for getTree() - defensive copy of session structure

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `entry` | `SessionEntry` |  |
| `children` | `SessionTreeNode[]` |  |
| `label` | `string or undefined` | Resolved label for this entry, if any |
| `labelTimestamp` | `string or undefined` | Timestamp of the latest label change for this entry, if any |


### :gear: SessionContext



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `messages` | `AgentMessage[]` |  |
| `thinkingLevel` | `string` |  |
| `model` | `{ provider: string; modelId: string; } or null` |  |


### :gear: SessionInfo



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `path` | `string` |  |
| `id` | `string` |  |
| `cwd` | `string` | Working directory where the session was started. Empty string for old sessions. |
| `name` | `string or undefined` | User-defined display name from session_info entries. |
| `parentSessionPath` | `string or undefined` | Path to the parent session (if this session was forked). |
| `created` | `Date` |  |
| `modified` | `Date` |  |
| `messageCount` | `number` |  |
| `firstMessage` | `string` |  |
| `allMessagesText` | `string` |  |


### :gear: FileOperations



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `read` | `Set<string>` |  |
| `written` | `Set<string>` |  |
| `edited` | `Set<string>` |  |


### :gear: CompactionDetails

Details stored in CompactionEntry.details for file tracking

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `readFiles` | `string[]` |  |
| `modifiedFiles` | `string[]` |  |


### :gear: CompactionResult

Result from compact() - SessionManager adds uuid/parentUuid when saving

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `summary` | `string` |  |
| `firstKeptEntryId` | `string` |  |
| `tokensBefore` | `number` |  |
| `details` | `T or undefined` | Extension-specific data (e.g., ArtifactIndex, version markers for structured compaction) |


### :gear: CompactionSettings



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `enabled` | `boolean` |  |
| `reserveTokens` | `number` |  |
| `keepRecentTokens` | `number` |  |


### :gear: ContextUsageEstimate



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `tokens` | `number` |  |
| `usageTokens` | `number` |  |
| `trailingTokens` | `number` |  |
| `lastUsageIndex` | `number or null` |  |


### :gear: CutPointResult



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `firstKeptEntryIndex` | `number` | Index of first entry to keep |
| `turnStartIndex` | `number` | Index of user message that starts the turn being split, or -1 if not splitting |
| `isSplitTurn` | `boolean` | Whether this cut splits a turn (cut point is not a user message) |


### :gear: CompactionPreparation



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `firstKeptEntryId` | `string` | UUID of first entry to keep |
| `messagesToSummarize` | `AgentMessage[]` | Messages that will be summarized and discarded |
| `turnPrefixMessages` | `AgentMessage[]` | Messages that will be turned into turn prefix summary (if splitting) |
| `isSplitTurn` | `boolean` | Whether this is a split turn (cut point in middle of turn) |
| `tokensBefore` | `number` |  |
| `previousSummary` | `string or undefined` | Summary from previous compaction, for iterative update |
| `fileOps` | `FileOperations` | File operations extracted from messagesToSummarize |
| `settings` | `CompactionSettings` | Compaction settions from settings.jsonl |


### :gear: BranchSummaryResult



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `summary` | `string or undefined` |  |
| `readFiles` | `string[] or undefined` |  |
| `modifiedFiles` | `string[] or undefined` |  |
| `aborted` | `boolean or undefined` |  |
| `error` | `string or undefined` |  |


### :gear: BranchSummaryDetails

Details stored in BranchSummaryEntry.details for file tracking

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `readFiles` | `string[]` |  |
| `modifiedFiles` | `string[]` |  |


### :gear: BranchPreparation



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `messages` | `AgentMessage[]` | Messages extracted for summarization, in chronological order |
| `fileOps` | `FileOperations` | File operations extracted from tool calls |
| `totalTokens` | `number` | Total estimated tokens in messages |


### :gear: CollectEntriesResult



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `entries` | `SessionEntry[]` | Entries to summarize, in chronological order |
| `commonAncestorId` | `string or null` | Common ancestor between old and new position, if any |


### :gear: GenerateBranchSummaryOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `model` | `Model<any>` | Model to use for summarization |
| `apiKey` | `string` | API key for the model |
| `headers` | `Record<string, string> or undefined` | Request headers for the model |
| `signal` | `AbortSignal` | Abort signal for cancellation |
| `customInstructions` | `string or undefined` | Optional custom instructions for summarization |
| `replaceInstructions` | `boolean or undefined` | If true, customInstructions replaces the default prompt instead of being appended |
| `reserveTokens` | `number or undefined` | Tokens reserved for prompt + LLM response (default 16384) |


### :gear: EventBus



| Property | Type | Description |
| ---------- | ---------- | ---------- |


### :gear: EventBusController



| Property | Type | Description |
| ---------- | ---------- | ---------- |


### :gear: ExecOptions

Options for executing shell commands.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `signal` | `AbortSignal or undefined` | AbortSignal to cancel the command |
| `timeout` | `number or undefined` | Timeout in milliseconds |
| `cwd` | `string or undefined` | Working directory |


### :gear: ExecResult

Result of executing a shell command.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `stdout` | `string` |  |
| `stderr` | `string` |  |
| `code` | `number` |  |
| `killed` | `boolean` |  |


### :gear: AppKeybindings



| Property | Type | Description |
| ---------- | ---------- | ---------- |


### :gear: AuthStorageBackend



| Property | Type | Description |
| ---------- | ---------- | ---------- |


### :gear: ProviderConfigInput

Input type for registerProvider API.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `name` | `string or undefined` |  |
| `baseUrl` | `string or undefined` |  |
| `apiKey` | `string or undefined` |  |
| `api` | `Api or undefined` |  |
| `streamSimple` | `((model: Model<Api>, context: Context, options?: SimpleStreamOptions or undefined) => AssistantMessageEventStream) or undefined` |  |
| `headers` | `Record<string, string> or undefined` |  |
| `authHeader` | `boolean or undefined` |  |
| `oauth` | `Omit<OAuthProviderInterface, "id"> or undefined` | OAuth provider for /login support |
| `models` | `{ id: string; name: string; api?: Api or undefined; baseUrl?: string or undefined; reasoning: boolean; thinkingLevelMap?: Partial<Record<ModelThinkingLevel, string or null>> or undefined; ... 5 more ...; compat?: OpenAICompletionsCompat or ... 2 more ... or undefined; }[] or undefined` |  |


### :gear: SlashCommandInfo



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `name` | `string` |  |
| `description` | `string or undefined` |  |
| `source` | `SlashCommandSource` |  |
| `sourceInfo` | `SourceInfo` |  |


### :gear: BuiltinSlashCommand



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `name` | `string` |  |
| `description` | `string` |  |


### :gear: ResourceCollision



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `name` | `string` |  |
| `winnerPath` | `string` |  |
| `loserPath` | `string` |  |
| `winnerSource` | `string or undefined` |  |
| `loserSource` | `string or undefined` |  |


### :gear: ResourceDiagnostic



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"error" or "warning" or "collision"` |  |
| `message` | `string` |  |
| `path` | `string or undefined` |  |
| `collision` | `ResourceCollision or undefined` |  |


### :gear: SkillFrontmatter



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `name` | `string or undefined` |  |
| `description` | `string or undefined` |  |


### :gear: Skill



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `name` | `string` |  |
| `description` | `string` |  |
| `filePath` | `string` |  |
| `baseDir` | `string` |  |
| `sourceInfo` | `SourceInfo` |  |
| `disableModelInvocation` | `boolean` |  |


### :gear: LoadSkillsResult



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `skills` | `Skill[]` |  |
| `diagnostics` | `ResourceDiagnostic[]` |  |


### :gear: LoadSkillsFromDirOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `dir` | `string` | Directory to scan for skills |
| `source` | `string` | Source identifier for these skills |


### :gear: LoadSkillsOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `cwd` | `string` | Working directory for project-local skills. |
| `agentDir` | `string` | Agent config directory for global skills. |
| `skillPaths` | `string[]` | Explicit skill paths (files or directories) |
| `includeDefaults` | `boolean` | Include default skills directories. |


### :gear: BuildSystemPromptOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `customPrompt` | `string or undefined` | 自定义系统提示（替换默认提示） |
| `selectedTools` | `string[] or undefined` | 选择的工具列表。默认: [read, bash, edit, write] |
| `toolSnippets` | `Record<string, string> or undefined` | 可选的工具摘要，以工具名为键 |
| `promptGuidelines` | `string[] or undefined` | 附加到默认系统提示准则的额外准则条目 |
| `appendSystemPrompt` | `string or undefined` | 追加到系统提示末尾的文本 |
| `cwd` | `string` | 工作目录路径 |
| `contextFiles` | `{ path: string; content: string; }[] or undefined` | 预加载的上下文文件 |
| `skills` | `Skill[] or undefined` | 预加载的技能列表 |


### :gear: RenderDiffOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `filePath` | `string or undefined` | File path (unused, kept for API compatibility) |


### :gear: FuzzyMatchResult



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `found` | `boolean` | Whether a match was found |
| `index` | `number` | The index where the match starts (in the content that should be used for replacement) |
| `matchLength` | `number` | Length of the matched text |
| `usedFuzzyMatch` | `boolean` | Whether fuzzy matching was used (false = exact match) |
| `contentForReplacement` | `string` | The content to use for replacement operations. When exact match: original content. When fuzzy match: normalized content. |


### :gear: Edit



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `oldText` | `string` |  |
| `newText` | `string` |  |


### :gear: AppliedEditsResult



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `baseContent` | `string` |  |
| `newContent` | `string` |  |


### :gear: EditDiffResult



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `diff` | `string` |  |
| `firstChangedLine` | `number or undefined` |  |


### :gear: EditDiffError



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `error` | `string` |  |


### :gear: EditToolDetails



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `diff` | `string` | Unified diff of the changes made |
| `firstChangedLine` | `number or undefined` | Line number of the first change in the new file (for editor navigation) |


### :gear: EditOperations

Pluggable operations for the edit tool.
Override these to delegate file editing to remote systems (for example SSH).

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `readFile` | `(absolutePath: string) => Promise<Buffer<ArrayBufferLike>>` | Read file contents as a Buffer |
| `writeFile` | `(absolutePath: string, content: string) => Promise<void>` | Write content to a file |
| `access` | `(absolutePath: string) => Promise<void>` | Check if file is readable and writable (throw if not) |


### :gear: EditToolOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `operations` | `EditOperations or undefined` | Custom operations for file editing. Default: local filesystem |


### :gear: FindToolDetails



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `truncation` | `TruncationResult or undefined` |  |
| `resultLimitReached` | `number or undefined` |  |


### :gear: FindOperations

Pluggable operations for the find tool.
Override these to delegate file search to remote systems (for example SSH).

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `exists` | `(absolutePath: string) => boolean or Promise<boolean>` | Check if path exists |
| `glob` | `(pattern: string, cwd: string, options: { ignore: string[]; limit: number; }) => string[] or Promise<string[]>` | Find files matching glob pattern. Returns relative or absolute paths. |


### :gear: FindToolOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `operations` | `FindOperations or undefined` | Custom operations for find. Default: local filesystem plus fd |


### :gear: GrepToolDetails



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `truncation` | `TruncationResult or undefined` |  |
| `matchLimitReached` | `number or undefined` |  |
| `linesTruncated` | `boolean or undefined` |  |


### :gear: GrepOperations

Pluggable operations for the grep tool.
Override these to delegate search to remote systems (for example SSH).

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `isDirectory` | `(absolutePath: string) => boolean or Promise<boolean>` | Check if path is a directory. Throws if path does not exist. |
| `readFile` | `(absolutePath: string) => string or Promise<string>` | Read file contents for context lines |


### :gear: GrepToolOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `operations` | `GrepOperations or undefined` | Custom operations for grep. Default: local filesystem plus ripgrep |


### :gear: LsToolDetails



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `truncation` | `TruncationResult or undefined` |  |
| `entryLimitReached` | `number or undefined` |  |


### :gear: LsOperations

Pluggable operations for the ls tool.
Override these to delegate directory listing to remote systems (for example SSH).

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `exists` | `(absolutePath: string) => boolean or Promise<boolean>` | Check if path exists |
| `stat` | `(absolutePath: string) => Promise<{ isDirectory: () => boolean; }> or { isDirectory: () => boolean; }` | Get file or directory stats. Throws if not found. |
| `readdir` | `(absolutePath: string) => string[] or Promise<string[]>` | Read directory entries |


### :gear: LsToolOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `operations` | `LsOperations or undefined` | Custom operations for directory listing. Default: local filesystem |


### :gear: ImageResizeOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `maxWidth` | `number or undefined` |  |
| `maxHeight` | `number or undefined` |  |
| `maxBytes` | `number or undefined` |  |
| `jpegQuality` | `number or undefined` |  |


### :gear: ResizedImage



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `data` | `string` |  |
| `mimeType` | `string` |  |
| `originalWidth` | `number` |  |
| `originalHeight` | `number` |  |
| `width` | `number` |  |
| `height` | `number` |  |
| `wasResized` | `boolean` |  |


### :gear: ReadToolDetails



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `truncation` | `TruncationResult or undefined` |  |


### :gear: ReadOperations

Pluggable operations for the read tool.
Override these to delegate file reading to remote systems (for example SSH).

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `readFile` | `(absolutePath: string) => Promise<Buffer<ArrayBufferLike>>` | Read file contents as a Buffer |
| `access` | `(absolutePath: string) => Promise<void>` | Check if file is readable (throw if not) |
| `detectImageMimeType` | `((absolutePath: string) => Promise<string or null or undefined>) or undefined` | Detect image MIME type, return null or undefined for non-images |


### :gear: ReadToolOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `autoResizeImages` | `boolean or undefined` | Whether to auto-resize images to 2000x2000 max. Default: true |
| `operations` | `ReadOperations or undefined` | Custom operations for file reading. Default: local filesystem |


### :gear: WriteOperations

Pluggable operations for the write tool.
Override these to delegate file writing to remote systems (for example SSH).

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `writeFile` | `(absolutePath: string, content: string) => Promise<void>` | Write content to a file |
| `mkdir` | `(dir: string) => Promise<void>` | Create directory recursively |


### :gear: WriteToolOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `operations` | `WriteOperations or undefined` | Custom operations for file writing. Default: local filesystem |


### :gear: ToolsOptions

各工具的配置选项聚合接口。
所有字段均为可选，未提供时使用各工具的默认配置。

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `read` | `ReadToolOptions or undefined` |  |
| `bash` | `BashToolOptions or undefined` |  |
| `write` | `WriteToolOptions or undefined` |  |
| `edit` | `EditToolOptions or undefined` |  |
| `grep` | `GrepToolOptions or undefined` |  |
| `find` | `FindToolOptions or undefined` |  |
| `ls` | `LsToolOptions or undefined` |  |


### :gear: ExtensionUIDialogOptions

Options for extension UI dialogs.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `signal` | `AbortSignal or undefined` | AbortSignal to programmatically dismiss the dialog. |
| `timeout` | `number or undefined` | Timeout in milliseconds. Dialog auto-dismisses with live countdown display. |


### :gear: ExtensionWidgetOptions

Options for extension widgets.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `placement` | `WidgetPlacement or undefined` | Where the widget is rendered. Defaults to "aboveEditor". |


### :gear: WorkingIndicatorOptions

Working indicator configuration for the interactive streaming loader.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `frames` | `string[] or undefined` | Animation frames. Use an empty array to hide the indicator entirely. Custom frames are rendered verbatim. |
| `intervalMs` | `number or undefined` | Frame interval in milliseconds for animated indicators. |


### :gear: ExtensionUIContext

UI context for extensions to request interactive UI.
Each mode (interactive, RPC, print) provides its own implementation.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `theme` | `Theme` | Get the current theme for styling. |


### :gear: ContextUsage



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `tokens` | `number or null` | Estimated context tokens, or null if unknown (e.g. right after compaction, before next LLM response). |
| `contextWindow` | `number` |  |
| `percent` | `number or null` | Context usage as percentage of context window, or null if tokens is unknown. |


### :gear: CompactOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `customInstructions` | `string or undefined` |  |
| `onComplete` | `((result: CompactionResult<unknown>) => void) or undefined` |  |
| `onError` | `((error: Error) => void) or undefined` |  |


### :gear: ExtensionContext

Context passed to extension event handlers.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `ui` | `ExtensionUIContext` | UI methods for user interaction |
| `hasUI` | `boolean` | Whether UI is available (false in print/RPC mode) |
| `cwd` | `string` | Current working directory |
| `sessionManager` | `ReadonlySessionManager` | Session manager (read-only) |
| `modelRegistry` | `ModelRegistry` | Model registry for API key resolution |
| `model` | `Model<any> or undefined` | Current model (may be undefined) |
| `signal` | `AbortSignal or undefined` | The current abort signal, or undefined when the agent is not streaming. |


### :gear: ExtensionCommandContext

Extended context for command handlers.
Includes session control methods only safe in user-initiated commands.

| Property | Type | Description |
| ---------- | ---------- | ---------- |


### :gear: ReplacedSessionContext

Fresh command-capable context bound to the replacement session after a session switch.

This is passed to `withSession()` callbacks on `newSession()`, `fork()`, and `switchSession()`.

| Property | Type | Description |
| ---------- | ---------- | ---------- |


### :gear: ToolRenderResultOptions

Rendering options for tool results

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `expanded` | `boolean` | Whether the result view is expanded |
| `isPartial` | `boolean` | Whether this is a partial/streaming result |


### :gear: ToolRenderContext

Context passed to tool renderers.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `args` | `TArgs` | Current tool call arguments. Shared across call/result renders for the same tool call. |
| `toolCallId` | `string` | Unique id for this tool execution. Stable across call/result renders for the same tool call. |
| `invalidate` | `() => void` | Invalidate just this tool execution component for redraw. |
| `lastComponent` | `Component or undefined` | Previously returned component for this render slot, if any. |
| `state` | `TState` | Shared renderer state for this tool row. Initialized by tool-execution.ts. |
| `cwd` | `string` | Working directory for this tool execution. |
| `executionStarted` | `boolean` | Whether the tool execution has started. |
| `argsComplete` | `boolean` | Whether the tool call arguments are complete. |
| `isPartial` | `boolean` | Whether the tool result is partial/streaming. |
| `expanded` | `boolean` | Whether the result view is expanded. |
| `showImages` | `boolean` | Whether inline images are currently shown in the TUI. |
| `isError` | `boolean` | Whether the current result is an error. |


### :gear: ToolDefinition

Tool definition for registerTool().

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `name` | `string` | Tool name (used in LLM tool calls) |
| `label` | `string` | Human-readable label for UI |
| `description` | `string` | Description for LLM |
| `promptSnippet` | `string or undefined` | Optional one-line snippet for the Available tools section in the default system prompt. Custom tools are omitted from that section when this is not provided. |
| `promptGuidelines` | `string[] or undefined` | Optional guideline bullets appended to the default system prompt Guidelines section when this tool is active. |
| `parameters` | `TParams` | Parameter schema (TypeBox) |
| `renderShell` | `"default" or "self" or undefined` | Controls whether ToolExecutionComponent renders the standard colored shell or the tool renders its own framing. |
| `prepareArguments` | `((args: unknown) => StaticType<[], "Encode", {}, {}, TParams>) or undefined` | Optional compatibility shim to prepare raw tool call arguments before schema validation. Must return an object conforming to TParams. |
| `executionMode` | `ToolExecutionMode or undefined` | Per-tool execution mode override. - "sequential": this tool must execute one at a time with other tool calls. - "parallel": this tool can execute concurrently with other tool calls.  If omitted, the default execution mode applies. |
| `renderCall` | `((args: StaticType<[], "Encode", {}, {}, TParams>, theme: Theme, context: ToolRenderContext<TState, StaticType<[], "Encode", {}, {}, TParams>>) => Component) or undefined` | Custom rendering for tool call display |
| `renderResult` | `((result: AgentToolResult<TDetails>, options: ToolRenderResultOptions, theme: Theme, context: ToolRenderContext<TState, StaticType<...>>) => Component) or undefined` | Custom rendering for tool result display |


### :gear: ResourcesDiscoverEvent

Fired after session_start to allow extensions to provide additional resource paths.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"resources_discover"` |  |
| `cwd` | `string` |  |
| `reason` | `"startup" or "reload"` |  |


### :gear: ResourcesDiscoverResult

Result from resources_discover event handler

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `skillPaths` | `string[] or undefined` |  |
| `promptPaths` | `string[] or undefined` |  |
| `themePaths` | `string[] or undefined` |  |


### :gear: SessionStartEvent

Fired when a session is started, loaded, or reloaded

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"session_start"` |  |
| `reason` | `"fork" or "startup" or "reload" or "new" or "resume"` | Why this session start happened. |
| `previousSessionFile` | `string or undefined` | Previously active session file. Present for "new", "resume", and "fork". |


### :gear: SessionBeforeSwitchEvent

Fired before switching to another session (can be cancelled)

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"session_before_switch"` |  |
| `reason` | `"new" or "resume"` |  |
| `targetSessionFile` | `string or undefined` |  |


### :gear: SessionBeforeForkEvent

Fired before forking a session (can be cancelled)

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"session_before_fork"` |  |
| `entryId` | `string` |  |
| `position` | `"before" or "at"` |  |


### :gear: SessionBeforeCompactEvent

Fired before context compaction (can be cancelled or customized)

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"session_before_compact"` |  |
| `preparation` | `CompactionPreparation` |  |
| `branchEntries` | `SessionEntry[]` |  |
| `customInstructions` | `string or undefined` |  |
| `signal` | `AbortSignal` |  |


### :gear: SessionCompactEvent

Fired after context compaction

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"session_compact"` |  |
| `compactionEntry` | `CompactionEntry<unknown>` |  |
| `fromExtension` | `boolean` |  |


### :gear: SessionShutdownEvent

Fired before an extension runtime is torn down due to quit, reload, or session replacement.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"session_shutdown"` |  |
| `reason` | `"fork" or "reload" or "new" or "resume" or "quit"` |  |
| `targetSessionFile` | `string or undefined` | Destination session file when shutting down due to session replacement. |


### :gear: TreePreparation

Preparation data for tree navigation

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `targetId` | `string` |  |
| `oldLeafId` | `string or null` |  |
| `commonAncestorId` | `string or null` |  |
| `entriesToSummarize` | `SessionEntry[]` |  |
| `userWantsSummary` | `boolean` |  |
| `customInstructions` | `string or undefined` | Custom instructions for summarization |
| `replaceInstructions` | `boolean or undefined` | If true, customInstructions replaces the default prompt instead of being appended |
| `label` | `string or undefined` | Label to attach to the branch summary entry |


### :gear: SessionBeforeTreeEvent

Fired before navigating in the session tree (can be cancelled)

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"session_before_tree"` |  |
| `preparation` | `TreePreparation` |  |
| `signal` | `AbortSignal` |  |


### :gear: SessionTreeEvent

Fired after navigating in the session tree

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"session_tree"` |  |
| `newLeafId` | `string or null` |  |
| `oldLeafId` | `string or null` |  |
| `summaryEntry` | `BranchSummaryEntry<unknown> or undefined` |  |
| `fromExtension` | `boolean or undefined` |  |


### :gear: ContextEvent

Fired before each LLM call. Can modify messages.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"context"` |  |
| `messages` | `AgentMessage[]` |  |


### :gear: BeforeProviderRequestEvent

Fired before a provider request is sent. Can replace the payload.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"before_provider_request"` |  |
| `payload` | `unknown` |  |


### :gear: AfterProviderResponseEvent

Fired after a provider response is received and before the response stream is consumed.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"after_provider_response"` |  |
| `status` | `number` |  |
| `headers` | `Record<string, string>` |  |


### :gear: BeforeAgentStartEvent

Fired after user submits prompt but before agent loop.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"before_agent_start"` |  |
| `prompt` | `string` | The raw user prompt text (after expansion). |
| `images` | `ImageContent[] or undefined` | Images attached to the user prompt, if any. |
| `systemPrompt` | `string` | The fully assembled system prompt string. |
| `systemPromptOptions` | `BuildSystemPromptOptions` | Structured options used to build the system prompt. Extensions can inspect this to understand what Pi loaded without re-discovering resources. |


### :gear: AgentStartEvent

Fired when an agent loop starts

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"agent_start"` |  |


### :gear: AgentEndEvent

Fired when an agent loop ends

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"agent_end"` |  |
| `messages` | `AgentMessage[]` |  |


### :gear: TurnStartEvent

Fired at the start of each turn

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"turn_start"` |  |
| `turnIndex` | `number` |  |
| `timestamp` | `number` |  |


### :gear: TurnEndEvent

Fired at the end of each turn

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"turn_end"` |  |
| `turnIndex` | `number` |  |
| `message` | `AgentMessage` |  |
| `toolResults` | `ToolResultMessage<any>[]` |  |


### :gear: MessageStartEvent

Fired when a message starts (user, assistant, or toolResult)

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"message_start"` |  |
| `message` | `AgentMessage` |  |


### :gear: MessageUpdateEvent

Fired during assistant message streaming with token-by-token updates

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"message_update"` |  |
| `message` | `AgentMessage` |  |
| `assistantMessageEvent` | `AssistantMessageEvent` |  |


### :gear: MessageEndEvent

Fired when a message ends

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"message_end"` |  |
| `message` | `AgentMessage` |  |


### :gear: ToolExecutionStartEvent

Fired when a tool starts executing

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"tool_execution_start"` |  |
| `toolCallId` | `string` |  |
| `toolName` | `string` |  |
| `args` | `any` |  |


### :gear: ToolExecutionUpdateEvent

Fired during tool execution with partial/streaming output

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"tool_execution_update"` |  |
| `toolCallId` | `string` |  |
| `toolName` | `string` |  |
| `args` | `any` |  |
| `partialResult` | `any` |  |


### :gear: ToolExecutionEndEvent

Fired when a tool finishes executing

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"tool_execution_end"` |  |
| `toolCallId` | `string` |  |
| `toolName` | `string` |  |
| `result` | `any` |  |
| `isError` | `boolean` |  |


### :gear: ModelSelectEvent

Fired when a new model is selected

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"model_select"` |  |
| `model` | `Model<any>` |  |
| `previousModel` | `Model<any> or undefined` |  |
| `source` | `ModelSelectSource` |  |


### :gear: ThinkingLevelSelectEvent

Fired when a new thinking level is selected

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"thinking_level_select"` |  |
| `level` | `ThinkingLevel` |  |
| `previousLevel` | `ThinkingLevel` |  |


### :gear: UserBashEvent

Fired when user executes a bash command via ! or !! prefix

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"user_bash"` |  |
| `command` | `string` | The command to execute |
| `excludeFromContext` | `boolean` | True if !! prefix was used (excluded from LLM context) |
| `cwd` | `string` | Current working directory |


### :gear: InputEvent

Fired when user input is received, before agent processing

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"input"` |  |
| `text` | `string` | The input text |
| `images` | `ImageContent[] or undefined` | Attached images, if any |
| `source` | `InputSource` | Where the input came from |


### :gear: BashToolCallEvent



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `toolName` | `"bash"` |  |
| `input` | `{ timeout?: number or undefined; command: string; }` |  |


### :gear: ReadToolCallEvent



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `toolName` | `"read"` |  |
| `input` | `{ limit?: number or undefined; offset?: number or undefined; path: string; }` |  |


### :gear: EditToolCallEvent



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `toolName` | `"edit"` |  |
| `input` | `{ path: string; edits: { oldText: string; newText: string; }[]; }` |  |


### :gear: WriteToolCallEvent



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `toolName` | `"write"` |  |
| `input` | `{ path: string; content: string; }` |  |


### :gear: GrepToolCallEvent



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `toolName` | `"grep"` |  |
| `input` | `{ path?: string or undefined; limit?: number or undefined; glob?: string or undefined; ignoreCase?: boolean or undefined; literal?: boolean or undefined; context?: number or undefined; pattern: string; }` |  |


### :gear: FindToolCallEvent



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `toolName` | `"find"` |  |
| `input` | `{ path?: string or undefined; limit?: number or undefined; pattern: string; }` |  |


### :gear: LsToolCallEvent



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `toolName` | `"ls"` |  |
| `input` | `{ path?: string or undefined; limit?: number or undefined; }` |  |


### :gear: CustomToolCallEvent



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `toolName` | `string` |  |
| `input` | `Record<string, unknown>` |  |


### :gear: BashToolResultEvent



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `toolName` | `"bash"` |  |
| `details` | `BashToolDetails or undefined` |  |


### :gear: ReadToolResultEvent



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `toolName` | `"read"` |  |
| `details` | `ReadToolDetails or undefined` |  |


### :gear: EditToolResultEvent



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `toolName` | `"edit"` |  |
| `details` | `EditToolDetails or undefined` |  |


### :gear: WriteToolResultEvent



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `toolName` | `"write"` |  |
| `details` | `undefined` |  |


### :gear: GrepToolResultEvent



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `toolName` | `"grep"` |  |
| `details` | `GrepToolDetails or undefined` |  |


### :gear: FindToolResultEvent



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `toolName` | `"find"` |  |
| `details` | `FindToolDetails or undefined` |  |


### :gear: LsToolResultEvent



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `toolName` | `"ls"` |  |
| `details` | `LsToolDetails or undefined` |  |


### :gear: CustomToolResultEvent



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `toolName` | `string` |  |
| `details` | `unknown` |  |


### :gear: ContextEventResult



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `messages` | `AgentMessage[] or undefined` |  |


### :gear: ToolCallEventResult



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `block` | `boolean or undefined` | Block tool execution. To modify arguments, mutate `event.input` in place instead. |
| `reason` | `string or undefined` |  |


### :gear: UserBashEventResult

Result from user_bash event handler

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `operations` | `BashOperations or undefined` | Custom operations to use for execution |
| `result` | `BashResult or undefined` | Full replacement: extension handled execution, use this result |


### :gear: ToolResultEventResult



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `content` | `(TextContent or ImageContent)[] or undefined` |  |
| `details` | `unknown` |  |
| `isError` | `boolean or undefined` |  |


### :gear: MessageEndEventResult



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `message` | `AgentMessage or undefined` | Replace the finalized message. The replacement must keep the original message role. |


### :gear: BeforeAgentStartEventResult



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `message` | `Pick<CustomMessage<unknown>, "content" or "customType" or "display" or "details"> or undefined` |  |
| `systemPrompt` | `string or undefined` | Replace the system prompt for this turn. If multiple extensions return this, they are chained. |


### :gear: SessionBeforeSwitchResult



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `cancel` | `boolean or undefined` |  |


### :gear: SessionBeforeForkResult



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `cancel` | `boolean or undefined` |  |
| `skipConversationRestore` | `boolean or undefined` |  |


### :gear: SessionBeforeCompactResult



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `cancel` | `boolean or undefined` |  |
| `compaction` | `CompactionResult<unknown> or undefined` |  |


### :gear: SessionBeforeTreeResult



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `cancel` | `boolean or undefined` |  |
| `summary` | `{ summary: string; details?: unknown; } or undefined` |  |
| `customInstructions` | `string or undefined` | Override custom instructions for summarization |
| `replaceInstructions` | `boolean or undefined` | Override whether customInstructions replaces the default prompt |
| `label` | `string or undefined` | Override label to attach to the branch summary entry |


### :gear: MessageRenderOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `expanded` | `boolean` |  |


### :gear: RegisteredCommand



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `name` | `string` |  |
| `sourceInfo` | `SourceInfo` |  |
| `description` | `string or undefined` |  |
| `getArgumentCompletions` | `((argumentPrefix: string) => AutocompleteItem[] or Promise<AutocompleteItem[] or null> or null) or undefined` |  |
| `handler` | `(args: string, ctx: ExtensionCommandContext) => Promise<void>` |  |


### :gear: ResolvedCommand



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `invocationName` | `string` |  |


### :gear: ExtensionAPI

ExtensionAPI passed to extension factory functions.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `events` | `EventBus` | Shared event bus for extension communication. |


### :gear: ProviderConfig

Configuration for registering a provider via pi.registerProvider().

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `name` | `string or undefined` | Display name for the provider in UI. |
| `baseUrl` | `string or undefined` | Base URL for the API endpoint. Required when defining models. |
| `apiKey` | `string or undefined` | API key or environment variable name. Required when defining models (unless oauth provided). |
| `api` | `Api or undefined` | API type. Required at provider or model level when defining models. |
| `streamSimple` | `((model: Model<Api>, context: Context, options?: SimpleStreamOptions or undefined) => AssistantMessageEventStream) or undefined` | Optional streamSimple handler for custom APIs. |
| `headers` | `Record<string, string> or undefined` | Custom headers to include in requests. |
| `authHeader` | `boolean or undefined` | If true, adds Authorization: Bearer header with the resolved API key. |
| `models` | `ProviderModelConfig[] or undefined` | Models to register. If provided, replaces all existing models for this provider. |
| `oauth` | `{ name: string; login(callbacks: OAuthLoginCallbacks): Promise<OAuthCredentials>; refreshToken(credentials: OAuthCredentials): Promise<...>; getApiKey(credentials: OAuthCredentials): string; modifyModels?(models: Model<...>[], credentials: OAuthCredentials): Model<...>[]; } or undefined` | OAuth provider for /login support. The `id` is set automatically from the provider name. |


### :gear: ProviderModelConfig

Configuration for a model within a provider.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `id` | `string` | Model ID (e.g., "claude-sonnet-4-20250514"). |
| `name` | `string` | Display name (e.g., "Claude 4 Sonnet"). |
| `api` | `Api or undefined` | API type override for this model. |
| `baseUrl` | `string or undefined` | API endpoint URL override for this model. |
| `reasoning` | `boolean` | Whether the model supports extended thinking. |
| `thinkingLevelMap` | `Partial<Record<ModelThinkingLevel, string or null>> or undefined` | Maps pi thinking levels to provider/model-specific values; null marks a level unsupported. |
| `input` | `("text" or "image")[]` | Supported input types. |
| `cost` | `{ input: number; output: number; cacheRead: number; cacheWrite: number; }` | Cost per token (for tracking, can be 0). |
| `contextWindow` | `number` | Maximum context window size in tokens. |
| `maxTokens` | `number` | Maximum output tokens. |
| `headers` | `Record<string, string> or undefined` | Custom headers for this model. |
| `compat` | `OpenAICompletionsCompat or OpenAIResponsesCompat or AnthropicMessagesCompat or undefined` | OpenAI compatibility settings. |


### :gear: RegisteredTool



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `definition` | `ToolDefinition<TSchema, unknown, any>` |  |
| `sourceInfo` | `SourceInfo` |  |


### :gear: ExtensionFlag



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `name` | `string` |  |
| `description` | `string or undefined` |  |
| `type` | `"string" or "boolean"` |  |
| `default` | `string or boolean or undefined` |  |
| `extensionPath` | `string` |  |


### :gear: ExtensionShortcut



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `shortcut` | `KeyId` |  |
| `description` | `string or undefined` |  |
| `handler` | `(ctx: ExtensionContext) => void or Promise<void>` |  |
| `extensionPath` | `string` |  |


### :gear: ExtensionRuntimeState

Shared state created by loader, used during registration and runtime.
Contains flag values (defaults set during registration, CLI values set after).

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `flagValues` | `Map<string, string or boolean>` |  |
| `pendingProviderRegistrations` | `{ name: string; config: ProviderConfig; extensionPath: string; }[]` | Provider registrations queued during extension loading, processed when runner binds |
| `assertActive` | `() => void` | Throws when this extension instance is stale after runtime replacement. |
| `invalidate` | `(message?: string or undefined) => void` | Marks this extension instance as stale after runtime replacement or reload. |
| `registerProvider` | `(name: string, config: ProviderConfig, extensionPath?: string or undefined) => void` | Register or unregister a provider.  Before bindCore(): queues registrations / removes from queue. After bindCore(): calls ModelRegistry directly for immediate effect. |
| `unregisterProvider` | `(name: string, extensionPath?: string or undefined) => void` |  |


### :gear: ExtensionActions

Action implementations for pi.* API methods.
Provided to runner.initialize(), copied into the shared runtime.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `sendMessage` | `SendMessageHandler` |  |
| `sendUserMessage` | `SendUserMessageHandler` |  |
| `appendEntry` | `AppendEntryHandler` |  |
| `setSessionName` | `SetSessionNameHandler` |  |
| `getSessionName` | `GetSessionNameHandler` |  |
| `setLabel` | `SetLabelHandler` |  |
| `getActiveTools` | `GetActiveToolsHandler` |  |
| `getAllTools` | `GetAllToolsHandler` |  |
| `setActiveTools` | `SetActiveToolsHandler` |  |
| `refreshTools` | `RefreshToolsHandler` |  |
| `getCommands` | `GetCommandsHandler` |  |
| `setModel` | `SetModelHandler` |  |
| `getThinkingLevel` | `GetThinkingLevelHandler` |  |
| `setThinkingLevel` | `SetThinkingLevelHandler` |  |


### :gear: ExtensionContextActions

Actions for ExtensionContext (ctx.* in event handlers).
Required by all modes.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `getModel` | `() => Model<any> or undefined` |  |
| `isIdle` | `() => boolean` |  |
| `getSignal` | `() => AbortSignal or undefined` |  |
| `abort` | `() => void` |  |
| `hasPendingMessages` | `() => boolean` |  |
| `shutdown` | `() => void` |  |
| `getContextUsage` | `() => ContextUsage or undefined` |  |
| `compact` | `(options?: CompactOptions or undefined) => void` |  |
| `getSystemPrompt` | `() => string` |  |


### :gear: ExtensionCommandContextActions

Actions for ExtensionCommandContext (ctx.* in command handlers).
Only needed for interactive mode where extension commands are invokable.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `waitForIdle` | `() => Promise<void>` |  |
| `newSession` | `(options?: { parentSession?: string or undefined; setup?: ((sessionManager: SessionManager) => Promise<void>) or undefined; withSession?: ((ctx: ReplacedSessionContext) => Promise<...>) or undefined; } or undefined) => Promise<...>` |  |
| `fork` | `(entryId: string, options?: { position?: "before" or "at" or undefined; withSession?: ((ctx: ReplacedSessionContext) => Promise<void>) or undefined; } or undefined) => Promise<...>` |  |
| `navigateTree` | `(targetId: string, options?: { summarize?: boolean or undefined; customInstructions?: string or undefined; replaceInstructions?: boolean or undefined; label?: string or undefined; } or undefined) => Promise<...>` |  |
| `switchSession` | `(sessionPath: string, options?: { withSession?: ((ctx: ReplacedSessionContext) => Promise<void>) or undefined; } or undefined) => Promise<{ cancelled: boolean; }>` |  |
| `reload` | `() => Promise<void>` |  |


### :gear: ExtensionRuntime

Full runtime = state + actions.
Created by loader with throwing action stubs, completed by runner.initialize().

| Property | Type | Description |
| ---------- | ---------- | ---------- |


### :gear: Extension

Loaded extension with all registered items.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `path` | `string` |  |
| `resolvedPath` | `string` |  |
| `sourceInfo` | `SourceInfo` |  |
| `handlers` | `Map<string, HandlerFn[]>` |  |
| `tools` | `Map<string, RegisteredTool>` |  |
| `messageRenderers` | `Map<string, MessageRenderer<unknown>>` |  |
| `commands` | `Map<string, RegisteredCommand>` |  |
| `flags` | `Map<string, ExtensionFlag>` |  |
| `shortcuts` | `Map<KeyId, ExtensionShortcut>` |  |


### :gear: LoadExtensionsResult

Result of loading extensions.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `extensions` | `Extension[]` |  |
| `errors` | `{ path: string; error: string; }[]` |  |
| `runtime` | `ExtensionRuntime` | Shared runtime - actions are throwing stubs until runner.initialize() |


### :gear: ExtensionError



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `extensionPath` | `string` |  |
| `event` | `string` |  |
| `error` | `string` |  |
| `stack` | `string or undefined` |  |


### :gear: Args



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `provider` | `string or undefined` |  |
| `model` | `string or undefined` |  |
| `apiKey` | `string or undefined` |  |
| `systemPrompt` | `string or undefined` |  |
| `appendSystemPrompt` | `string[] or undefined` |  |
| `thinking` | `ThinkingLevel or undefined` |  |
| `continue` | `boolean or undefined` |  |
| `resume` | `boolean or undefined` |  |
| `help` | `boolean or undefined` |  |
| `version` | `boolean or undefined` |  |
| `mode` | `Mode or undefined` |  |
| `noSession` | `boolean or undefined` |  |
| `session` | `string or undefined` |  |
| `fork` | `string or undefined` |  |
| `sessionDir` | `string or undefined` |  |
| `models` | `string[] or undefined` |  |
| `tools` | `string[] or undefined` |  |
| `noTools` | `boolean or undefined` |  |
| `noBuiltinTools` | `boolean or undefined` |  |
| `extensions` | `string[] or undefined` |  |
| `noExtensions` | `boolean or undefined` |  |
| `print` | `boolean or undefined` |  |
| `export` | `string or undefined` |  |
| `noSkills` | `boolean or undefined` |  |
| `skills` | `string[] or undefined` |  |
| `promptTemplates` | `string[] or undefined` |  |
| `noPromptTemplates` | `boolean or undefined` |  |
| `themes` | `string[] or undefined` |  |
| `noThemes` | `boolean or undefined` |  |
| `noContextFiles` | `boolean or undefined` |  |
| `listModels` | `string or true or undefined` |  |
| `offline` | `boolean or undefined` |  |
| `verbose` | `boolean or undefined` |  |
| `messages` | `string[]` |  |
| `fileArgs` | `string[]` |  |
| `unknownFlags` | `Map<string, string or boolean>` | Unknown flags (potentially extension flags) - map of flag name to value |
| `diagnostics` | `{ type: "error" or "warning"; message: string; }[]` |  |


### :gear: ProcessedFiles



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `text` | `string` |  |
| `images` | `ImageContent[]` |  |


### :gear: ProcessFileOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `autoResizeImages` | `boolean or undefined` | Whether to auto-resize images to 2000x2000 max. Default: true |


### :gear: InitialMessageInput



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `parsed` | `Args` |  |
| `fileText` | `string or undefined` |  |
| `fileImages` | `ImageContent[] or undefined` |  |
| `stdinContent` | `string or undefined` |  |


### :gear: InitialMessageResult



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `initialMessage` | `string or undefined` |  |
| `initialImages` | `ImageContent[] or undefined` |  |


### :gear: ParsedSearchQuery



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `mode` | `"tokens" or "regex"` |  |
| `tokens` | `{ kind: "fuzzy" or "phrase"; value: string; }[]` |  |
| `regex` | `RegExp or null` |  |
| `error` | `string or undefined` | If set, parsing failed and we should treat query as non-matching. |


### :gear: MatchResult



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `matches` | `boolean` |  |
| `score` | `number` | Lower is better; only meaningful when matches === true |


### :gear: ToolHtmlRenderer

Interface for rendering custom tools to HTML.
Used by agent-session to pre-render extension tool output.

| Property | Type | Description |
| ---------- | ---------- | ---------- |


### :gear: ExportOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `outputPath` | `string or undefined` |  |
| `themeName` | `string or undefined` |  |
| `toolRenderer` | `ToolHtmlRenderer or undefined` | Optional tool renderer for custom tools |


### :gear: ToolHtmlRendererDeps



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `getToolDefinition` | `(name: string) => ToolDefinition<TSchema, unknown, any> or undefined` | Function to look up tool definition by name |
| `theme` | `Theme` | Theme for styling |
| `cwd` | `string` | Working directory for render context |
| `width` | `number or undefined` | Terminal width for rendering (default: 100) |


### :gear: ToolHtmlRenderer



| Property | Type | Description |
| ---------- | ---------- | ---------- |


### :gear: PromptTemplate

Represents a prompt template loaded from a markdown file

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `name` | `string` |  |
| `description` | `string` |  |
| `argumentHint` | `string or undefined` |  |
| `content` | `string` |  |
| `sourceInfo` | `SourceInfo` |  |
| `filePath` | `string` |  |


### :gear: LoadPromptTemplatesOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `cwd` | `string` | Working directory for project-local templates. |
| `agentDir` | `string` | Agent config directory for global templates. |
| `promptPaths` | `string[]` | Explicit prompt template paths (files or directories). |
| `includeDefaults` | `boolean` | Include default prompt directories. |


### :gear: ResourceExtensionPaths



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `skillPaths` | `{ path: string; metadata: PathMetadata; }[] or undefined` |  |
| `promptPaths` | `{ path: string; metadata: PathMetadata; }[] or undefined` |  |
| `themePaths` | `{ path: string; metadata: PathMetadata; }[] or undefined` |  |


### :gear: ResourceLoader



| Property | Type | Description |
| ---------- | ---------- | ---------- |


### :gear: DefaultResourceLoaderOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `cwd` | `string` |  |
| `agentDir` | `string` |  |
| `settingsManager` | `SettingsManager or undefined` |  |
| `eventBus` | `EventBus or undefined` |  |
| `additionalExtensionPaths` | `string[] or undefined` |  |
| `additionalSkillPaths` | `string[] or undefined` |  |
| `additionalPromptTemplatePaths` | `string[] or undefined` |  |
| `additionalThemePaths` | `string[] or undefined` |  |
| `extensionFactories` | `ExtensionFactory[] or undefined` |  |
| `noExtensions` | `boolean or undefined` |  |
| `noSkills` | `boolean or undefined` |  |
| `noPromptTemplates` | `boolean or undefined` |  |
| `noThemes` | `boolean or undefined` |  |
| `noContextFiles` | `boolean or undefined` |  |
| `systemPrompt` | `string or undefined` |  |
| `appendSystemPrompt` | `string[] or undefined` |  |
| `extensionsOverride` | `((base: LoadExtensionsResult) => LoadExtensionsResult) or undefined` |  |
| `skillsOverride` | `((base: { skills: Skill[]; diagnostics: ResourceDiagnostic[]; }) => { skills: Skill[]; diagnostics: ResourceDiagnostic[]; }) or undefined` |  |
| `promptsOverride` | `((base: { prompts: PromptTemplate[]; diagnostics: ResourceDiagnostic[]; }) => { prompts: PromptTemplate[]; diagnostics: ResourceDiagnostic[]; }) or undefined` |  |
| `themesOverride` | `((base: { themes: Theme[]; diagnostics: ResourceDiagnostic[]; }) => { themes: Theme[]; diagnostics: ResourceDiagnostic[]; }) or undefined` |  |
| `agentsFilesOverride` | `((base: { agentsFiles: { path: string; content: string; }[]; }) => { agentsFiles: { path: string; content: string; }[]; }) or undefined` |  |
| `systemPromptOverride` | `((base: string or undefined) => string or undefined) or undefined` |  |
| `appendSystemPromptOverride` | `((base: string[]) => string[]) or undefined` |  |


### :gear: ScopedModel

作用域限定后的模型，包含可选的思维级别

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `model` | `Model<Api>` |  |
| `thinkingLevel` | `ThinkingLevel or undefined` | 从模式中显式指定的思维级别（如 "model:high"），未指定则为 undefined |


### :gear: ParsedModelResult

模型模式解析结果

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `model` | `Model<Api> or undefined` |  |
| `thinkingLevel` | `ThinkingLevel or undefined` | 从模式中解析出的思维级别，未指定则为 undefined |
| `warning` | `string or undefined` |  |


### :gear: ResolveCliModelResult

CLI 模型解析结果

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `model` | `Model<Api> or undefined` |  |
| `thinkingLevel` | `ThinkingLevel or undefined` |  |
| `warning` | `string or undefined` |  |
| `error` | `string or undefined` | 适合 CLI 显示的错误信息。设置时 model 为 undefined |


### :gear: InitialModelResult

初始模型选择结果

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `model` | `Model<Api> or undefined` |  |
| `thinkingLevel` | `ThinkingLevel` |  |
| `fallbackMessage` | `string or undefined` |  |


### :gear: CreateAgentSessionOptions

创建代理会话的配置选项

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `cwd` | `string or undefined` | 项目工作目录，用于本地发现配置文件。默认值：process.cwd() |
| `agentDir` | `string or undefined` | 全局配置目录。默认值：~/.pi/agent |
| `authStorage` | `AuthStorage or undefined` | 认证凭证存储。默认值：AuthStorage.create(agentDir/auth.json) |
| `modelRegistry` | `ModelRegistry or undefined` | 模型注册表。默认值：ModelRegistry.create(authStorage, agentDir/models.json) |
| `model` | `Model<any> or undefined` | 要使用的模型。默认值：依次从设置、注册表中查找 |
| `thinkingLevel` | `ThinkingLevel or undefined` | 思考深度等级。默认值：从设置获取，否则为 'medium'（按模型能力钳位） |
| `scopedModels` | `{ model: Model<any>; thinkingLevel?: ThinkingLevel or undefined; }[] or undefined` | 可用于循环切换的模型列表（交互模式下 Ctrl+P 触发） |
| `noTools` | `"all" or "builtin" or undefined` | 可选的默认工具抑制模式，当未提供显式白名单时生效。  - "all"：不启用任何工具 - "builtin"：禁用默认内置工具（read、bash、edit、write）， 但保留扩展/自定义工具 |
| `tools` | `string[] or undefined` | 可选的工具名称白名单。  省略时启用默认内置工具（read、bash、edit、write）， 扩展/自定义工具也保持启用，除非 `noTools` 改变了默认行为。 提供时，仅启用列表中的工具。 |
| `customTools` | `ToolDefinition<TSchema, unknown, any>[] or undefined` | 要注册的自定义工具（在内置工具之外额外添加） |
| `resourceLoader` | `ResourceLoader or undefined` | 资源加载器。省略时使用 DefaultResourceLoader |
| `sessionManager` | `SessionManager or undefined` | 会话管理器。默认值：SessionManager.create(cwd) |
| `settingsManager` | `SettingsManager or undefined` | 设置管理器。默认值：SettingsManager.create(cwd, agentDir) |
| `sessionStartEvent` | `SessionStartEvent or undefined` | 会话启动事件元数据，传递给扩展运行时以完成启动初始化 |


### :gear: CreateAgentSessionResult

createAgentSession 的返回结果

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `session` | `AgentSession` | 创建的代理会话实例 |
| `extensionsResult` | `LoadExtensionsResult` | 扩展加载结果（用于交互模式下的 UI 上下文设置） |
| `modelFallbackMessage` | `string or undefined` | 当恢复的会话使用了与保存时不同的模型时产生的警告信息 |


### :gear: SessionCwdIssue



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `sessionFile` | `string or undefined` |  |
| `sessionCwd` | `string` |  |
| `fallbackCwd` | `string` |  |


### :gear: ChangelogEntry



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `major` | `number` |  |
| `minor` | `number` |  |
| `patch` | `number` |  |
| `content` | `string` |  |


### :gear: LatestPiRelease



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `version` | `string` |  |
| `packageName` | `string or undefined` |  |


### :gear: ExtensionInputOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `tui` | `TUI or undefined` |  |
| `timeout` | `number or undefined` |  |


### :gear: ExtensionSelectorOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `tui` | `TUI or undefined` |  |
| `timeout` | `number or undefined` |  |


### :gear: ModelsConfig



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `allModels` | `Model<any>[]` |  |
| `enabledModelIds` | `string[] or null` |  |


### :gear: ModelsCallbacks



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `onChange` | `(enabledModelIds: string[] or null) => void or Promise<void>` | Called whenever the enabled model set or order changes (session-only, no persist) |
| `onPersist` | `(enabledModelIds: string[] or null) => void or Promise<void>` | Called when user wants to persist current selection to settings |
| `onCancel` | `() => void` |  |


### :gear: SettingsConfig



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `autoCompact` | `boolean` |  |
| `showImages` | `boolean` |  |
| `imageWidthCells` | `number` |  |
| `autoResizeImages` | `boolean` |  |
| `blockImages` | `boolean` |  |
| `enableSkillCommands` | `boolean` |  |
| `steeringMode` | `"all" or "one-at-a-time"` |  |
| `followUpMode` | `"all" or "one-at-a-time"` |  |
| `transport` | `Transport` |  |
| `thinkingLevel` | `ThinkingLevel` |  |
| `availableThinkingLevels` | `ThinkingLevel[]` |  |
| `currentTheme` | `string` |  |
| `availableThemes` | `string[]` |  |
| `hideThinkingBlock` | `boolean` |  |
| `collapseChangelog` | `boolean` |  |
| `enableInstallTelemetry` | `boolean` |  |
| `doubleEscapeAction` | `"fork" or "tree" or "none"` |  |
| `treeFilterMode` | `"all" or "default" or "no-tools" or "user-only" or "labeled-only"` |  |
| `showHardwareCursor` | `boolean` |  |
| `editorPaddingX` | `number` |  |
| `autocompleteMaxVisible` | `number` |  |
| `quietStartup` | `boolean` |  |
| `clearOnShrink` | `boolean` |  |
| `showTerminalProgress` | `boolean` |  |
| `warnings` | `WarningSettings` |  |


### :gear: SettingsCallbacks



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `onAutoCompactChange` | `(enabled: boolean) => void` |  |
| `onShowImagesChange` | `(enabled: boolean) => void` |  |
| `onImageWidthCellsChange` | `(width: number) => void` |  |
| `onAutoResizeImagesChange` | `(enabled: boolean) => void` |  |
| `onBlockImagesChange` | `(blocked: boolean) => void` |  |
| `onEnableSkillCommandsChange` | `(enabled: boolean) => void` |  |
| `onSteeringModeChange` | `(mode: "all" or "one-at-a-time") => void` |  |
| `onFollowUpModeChange` | `(mode: "all" or "one-at-a-time") => void` |  |
| `onTransportChange` | `(transport: Transport) => void` |  |
| `onThinkingLevelChange` | `(level: ThinkingLevel) => void` |  |
| `onThemeChange` | `(theme: string) => void` |  |
| `onThemePreview` | `((theme: string) => void) or undefined` |  |
| `onHideThinkingBlockChange` | `(hidden: boolean) => void` |  |
| `onCollapseChangelogChange` | `(collapsed: boolean) => void` |  |
| `onEnableInstallTelemetryChange` | `(enabled: boolean) => void` |  |
| `onDoubleEscapeActionChange` | `(action: "fork" or "tree" or "none") => void` |  |
| `onTreeFilterModeChange` | `(mode: "all" or "default" or "no-tools" or "user-only" or "labeled-only") => void` |  |
| `onShowHardwareCursorChange` | `(enabled: boolean) => void` |  |
| `onEditorPaddingXChange` | `(padding: number) => void` |  |
| `onAutocompleteMaxVisibleChange` | `(maxVisible: number) => void` |  |
| `onQuietStartupChange` | `(enabled: boolean) => void` |  |
| `onClearOnShrinkChange` | `(enabled: boolean) => void` |  |
| `onShowTerminalProgressChange` | `(enabled: boolean) => void` |  |
| `onWarningsChange` | `(warnings: WarningSettings) => void` |  |
| `onCancel` | `() => void` |  |


### :gear: ToolExecutionOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `showImages` | `boolean or undefined` |  |
| `imageWidthCells` | `number or undefined` |  |


### :gear: InteractiveModeOptions

交互模式初始化选项。
控制启动时的迁移警告、模型回退提示、初始消息等行为。

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `migratedProviders` | `string[] or undefined` | 已迁移到 auth.json 的提供商列表（显示迁移提示） |
| `modelFallbackMessage` | `string or undefined` | 当会话模型无法恢复时的警告消息（如原模型已不可用） |
| `initialMessage` | `string or undefined` | 启动时发送的初始消息（可包含 file: 内容） |
| `initialImages` | `ImageContent[] or undefined` | 附加到初始消息的图片 |
| `initialMessages` | `string[] or undefined` | 在初始消息后继续发送的附加消息 |
| `verbose` | `boolean or undefined` | 强制显示启动详情（覆盖 quietStartup 设置） |


### :gear: PrintModeOptions

Options for print mode.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `mode` | `"text" or "json"` | Output mode: "text" for final response only, "json" for all events |
| `messages` | `string[] or undefined` | Array of additional prompts to send after initialMessage |
| `initialMessage` | `string or undefined` | First message to send (may contain file: content) |
| `initialImages` | `ImageContent[] or undefined` | Images to attach to the initial message |


### :gear: RpcSlashCommand

A command available for invocation via prompt

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `name` | `string` | Command name (without leading slash) |
| `description` | `string or undefined` | Human-readable description |
| `source` | `"extension" or "prompt" or "skill"` | What kind of command this is |
| `sourceInfo` | `SourceInfo` | Source metadata for the owning resource |


### :gear: RpcSessionState



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `model` | `Model<any> or undefined` |  |
| `thinkingLevel` | `ThinkingLevel` |  |
| `isStreaming` | `boolean` |  |
| `isCompacting` | `boolean` |  |
| `steeringMode` | `"all" or "one-at-a-time"` |  |
| `followUpMode` | `"all" or "one-at-a-time"` |  |
| `sessionFile` | `string or undefined` |  |
| `sessionId` | `string` |  |
| `sessionName` | `string or undefined` |  |
| `autoCompactionEnabled` | `boolean` |  |
| `messageCount` | `number` |  |
| `pendingMessageCount` | `number` |  |


### :gear: RpcClientOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `cliPath` | `string or undefined` | Path to the CLI entry point (default: searches for dist/cli.js) |
| `cwd` | `string or undefined` | Working directory for the agent |
| `env` | `Record<string, string> or undefined` | Environment variables |
| `provider` | `string or undefined` | Provider to use |
| `model` | `string or undefined` | Model ID to use |
| `args` | `string[] or undefined` | Additional CLI arguments |


### :gear: ModelInfo



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `provider` | `string` |  |
| `id` | `string` |  |
| `contextWindow` | `number` |  |
| `reasoning` | `boolean` |  |


### :gear: ParsedSkillBlock

从用户消息中解析出的技能块结构

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `name` | `string` |  |
| `location` | `string` |  |
| `content` | `string` |  |
| `userMessage` | `string or undefined` |  |


### :gear: AgentSessionConfig

Agent 会话配置选项

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `agent` | `Agent` |  |
| `sessionManager` | `SessionManager` |  |
| `settingsManager` | `SettingsManager` |  |
| `cwd` | `string` |  |
| `scopedModels` | `{ model: Model<any>; thinkingLevel?: ThinkingLevel or undefined; }[] or undefined` | 通过 Ctrl+P 循环切换的模型列表（来自 --models 参数） |
| `resourceLoader` | `ResourceLoader` | 资源加载器，用于加载技能、提示模板、主题、对话上下文文件和系统提示 |
| `customTools` | `ToolDefinition<TSchema, unknown, any>[] or undefined` | 通过 SDK 注册的自定义工具（非扩展注册） |
| `modelRegistry` | `ModelRegistry` | 模型注册表，用于 API 密钥解析和模型发现 |
| `initialActiveToolNames` | `string[] or undefined` | 初始激活的内置工具名称列表，默认为 [read, bash, edit, write] |
| `allowedToolNames` | `string[] or undefined` | 工具名称白名单，设置后仅暴露白名单中的工具 |
| `baseToolsOverride` | `Record<string, AgentTool<TSchema, any>> or undefined` | 覆盖基础工具集（适用于自定义运行时）。  内部会合成为最小化的 ToolDefinition，使 AgentSession 即使在调用方提供纯 AgentTool 实例时也能维持定义优先的工具注册表。 |
| `extensionRunnerRef` | `{ current?: ExtensionRunner or undefined; } or undefined` | Agent 用于访问当前 ExtensionRunner 的可变引用 |
| `sessionStartEvent` | `SessionStartEvent or undefined` | 扩展绑定到运行时时发出的会话启动事件元数据 |


### :gear: ExtensionBindings

扩展绑定配置，用于将 UI 上下文、命令操作、关闭处理和错误监听器绑定到会话

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `uiContext` | `ExtensionUIContext or undefined` |  |
| `commandContextActions` | `ExtensionCommandContextActions or undefined` |  |
| `shutdownHandler` | `ShutdownHandler or undefined` |  |
| `onError` | `ExtensionErrorListener or undefined` |  |


### :gear: PromptOptions

Options for AgentSession.prompt()

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `expandPromptTemplates` | `boolean or undefined` | Whether to expand file-based prompt templates (default: true) |
| `images` | `ImageContent[] or undefined` | Image attachments |
| `streamingBehavior` | `"steer" or "followUp" or undefined` | When streaming, how to queue the message: "steer" (interrupt) or "followUp" (wait). Required if streaming. |
| `source` | `InputSource or undefined` | Source of input for extension input event handlers. Defaults to "interactive". |
| `preflightResult` | `((success: boolean) => void) or undefined` | Internal hook used by RPC mode to observe prompt preflight acceptance or rejection. |


### :gear: ModelCycleResult

Result from cycleModel()

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `model` | `Model<any>` |  |
| `thinkingLevel` | `ThinkingLevel` |  |
| `isScoped` | `boolean` | Whether cycling through scoped models (--models flag) or all available |


### :gear: SessionStats

Session statistics for /session command

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `sessionFile` | `string or undefined` |  |
| `sessionId` | `string` |  |
| `userMessages` | `number` |  |
| `assistantMessages` | `number` |  |
| `toolCalls` | `number` |  |
| `toolResults` | `number` |  |
| `totalMessages` | `number` |  |
| `tokens` | `{ input: number; output: number; cacheRead: number; cacheWrite: number; total: number; }` |  |
| `cost` | `number` |  |
| `contextUsage` | `ContextUsage or undefined` |  |


### :gear: AgentSessionRuntimeDiagnostic

Non-fatal issues collected while creating services or sessions.

Runtime creation returns diagnostics to the caller instead of printing or
exiting. The app layer decides whether warnings should be shown and whether
errors should abort startup.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"error" or "warning" or "info"` |  |
| `message` | `string` |  |


### :gear: CreateAgentSessionServicesOptions

Inputs for creating cwd-bound runtime services.

These services are recreated whenever the effective session cwd changes.
CLI-provided resource paths should be resolved to absolute paths before they
reach this function, so later cwd switches do not reinterpret them.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `cwd` | `string` |  |
| `agentDir` | `string or undefined` |  |
| `authStorage` | `AuthStorage or undefined` |  |
| `settingsManager` | `SettingsManager or undefined` |  |
| `modelRegistry` | `ModelRegistry or undefined` |  |
| `extensionFlagValues` | `Map<string, string or boolean> or undefined` |  |
| `resourceLoaderOptions` | `Omit<DefaultResourceLoaderOptions, "cwd" or "agentDir" or "settingsManager"> or undefined` |  |


### :gear: CreateAgentSessionFromServicesOptions

Inputs for creating an AgentSession from already-created services.

Use this after services exist and any cwd-bound model/tool/session options
have been resolved against those services.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `services` | `AgentSessionServices` |  |
| `sessionManager` | `SessionManager` |  |
| `sessionStartEvent` | `SessionStartEvent or undefined` |  |
| `model` | `Model<any> or undefined` |  |
| `thinkingLevel` | `ThinkingLevel or undefined` |  |
| `scopedModels` | `{ model: Model<any>; thinkingLevel?: ThinkingLevel or undefined; }[] or undefined` |  |
| `tools` | `string[] or undefined` |  |
| `noTools` | `"all" or "builtin" or undefined` |  |
| `customTools` | `ToolDefinition<TSchema, unknown, any>[] or undefined` |  |


### :gear: AgentSessionServices

Coherent cwd-bound runtime services for one effective session cwd.

This is infrastructure only. The AgentSession itself is created separately so
session options can be resolved against these services first.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `cwd` | `string` |  |
| `agentDir` | `string` |  |
| `authStorage` | `AuthStorage` |  |
| `settingsManager` | `SettingsManager` |  |
| `modelRegistry` | `ModelRegistry` |  |
| `resourceLoader` | `ResourceLoader` |  |
| `diagnostics` | `AgentSessionRuntimeDiagnostic[]` |  |


### :gear: CreateAgentSessionRuntimeResult

Result returned by runtime creation.

The caller gets the created session, its cwd-bound services, and all
diagnostics collected during setup.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `services` | `AgentSessionServices` |  |
| `diagnostics` | `AgentSessionRuntimeDiagnostic[]` |  |


### :gear: ConfigSelectorOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `resolvedPaths` | `ResolvedPaths` |  |
| `settingsManager` | `SettingsManager` |  |
| `cwd` | `string` |  |
| `agentDir` | `string` |  |


### :gear: MainOptions

主函数的可选配置。
允许外部调用者注入自定义的扩展工厂。

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `extensionFactories` | `ExtensionFactory[] or undefined` |  |


## :cocktail: Types

- [InstallMethod](#gear-installmethod)
- [GitSource](#gear-gitsource)
- [TransportSetting](#gear-transportsetting)
- [PackageSource](#gear-packagesource)
- [SettingsScope](#gear-settingsscope)
- [MissingSourceAction](#gear-missingsourceaction)
- [ProgressCallback](#gear-progresscallback)
- [SourceScope](#gear-sourcescope)
- [SourceOrigin](#gear-sourceorigin)
- [ThemeColor](#gear-themecolor)
- [ThemeBg](#gear-themebg)
- [ToolRenderResultLike](#gear-toolrenderresultlike)
- [BashToolInput](#gear-bashtoolinput)
- [BashSpawnHook](#gear-bashspawnhook)
- [SessionEntry](#gear-sessionentry)
- [FileEntry](#gear-fileentry)
- [ReadonlySessionManager](#gear-readonlysessionmanager)
- [SessionListProgress](#gear-sessionlistprogress)
- [ReadonlyFooterDataProvider](#gear-readonlyfooterdataprovider)
- [AppKeybinding](#gear-appkeybinding)
- [ApiKeyCredential](#gear-apikeycredential)
- [OAuthCredential](#gear-oauthcredential)
- [AuthCredential](#gear-authcredential)
- [AuthStorageData](#gear-authstoragedata)
- [AuthStatus](#gear-authstatus)
- [ResolvedRequestAuth](#gear-resolvedrequestauth)
- [SlashCommandSource](#gear-slashcommandsource)
- [EditToolInput](#gear-edittoolinput)
- [FindToolInput](#gear-findtoolinput)
- [GrepToolInput](#gear-greptoolinput)
- [LsToolInput](#gear-lstoolinput)
- [ReadToolInput](#gear-readtoolinput)
- [WriteToolInput](#gear-writetoolinput)
- [Tool](#gear-tool)
- [ToolDef](#gear-tooldef)
- [ToolName](#gear-toolname)
- [WidgetPlacement](#gear-widgetplacement)
- [TerminalInputHandler](#gear-terminalinputhandler)
- [AutocompleteProviderFactory](#gear-autocompleteproviderfactory)
- [EditorFactory](#gear-editorfactory)
- [SessionEvent](#gear-sessionevent)
- [ModelSelectSource](#gear-modelselectsource)
- [InputSource](#gear-inputsource)
- [InputEventResult](#gear-inputeventresult)
- [ToolCallEvent](#gear-toolcallevent)
- [ToolResultEvent](#gear-toolresultevent)
- [ExtensionEvent](#gear-extensionevent)
- [BeforeProviderRequestEventResult](#gear-beforeproviderrequesteventresult)
- [MessageRenderer](#gear-messagerenderer)
- [ExtensionHandler](#gear-extensionhandler)
- [ExtensionFactory](#gear-extensionfactory)
- [SendMessageHandler](#gear-sendmessagehandler)
- [SendUserMessageHandler](#gear-sendusermessagehandler)
- [AppendEntryHandler](#gear-appendentryhandler)
- [SetSessionNameHandler](#gear-setsessionnamehandler)
- [GetSessionNameHandler](#gear-getsessionnamehandler)
- [GetActiveToolsHandler](#gear-getactivetoolshandler)
- [ToolInfo](#gear-toolinfo)
- [GetAllToolsHandler](#gear-getalltoolshandler)
- [GetCommandsHandler](#gear-getcommandshandler)
- [SetActiveToolsHandler](#gear-setactivetoolshandler)
- [RefreshToolsHandler](#gear-refreshtoolshandler)
- [SetModelHandler](#gear-setmodelhandler)
- [GetThinkingLevelHandler](#gear-getthinkinglevelhandler)
- [SetThinkingLevelHandler](#gear-setthinkinglevelhandler)
- [SetLabelHandler](#gear-setlabelhandler)
- [Mode](#gear-mode)
- [SortMode](#gear-sortmode)
- [NameFilter](#gear-namefilter)
- [ClipboardModule](#gear-clipboardmodule)
- [ClipboardImage](#gear-clipboardimage)
- [AuthSelectorProvider](#gear-authselectorprovider)
- [FilterMode](#gear-filtermode)
- [RpcCommand](#gear-rpccommand)
- [RpcResponse](#gear-rpcresponse)
- [RpcExtensionUIRequest](#gear-rpcextensionuirequest)
- [RpcExtensionUIResponse](#gear-rpcextensionuiresponse)
- [RpcCommandType](#gear-rpccommandtype)
- [RpcEventListener](#gear-rpceventlistener)
- [ExtensionErrorListener](#gear-extensionerrorlistener)
- [NewSessionHandler](#gear-newsessionhandler)
- [ForkHandler](#gear-forkhandler)
- [NavigateTreeHandler](#gear-navigatetreehandler)
- [SwitchSessionHandler](#gear-switchsessionhandler)
- [ReloadHandler](#gear-reloadhandler)
- [ShutdownHandler](#gear-shutdownhandler)
- [AgentSessionEvent](#gear-agentsessionevent)
- [AgentSessionEventListener](#gear-agentsessioneventlistener)
- [CreateAgentSessionRuntimeFactory](#gear-createagentsessionruntimefactory)
- [PackageCommand](#gear-packagecommand)

### :gear: InstallMethod

安装方式类型，表示包是通过哪种包管理器安装的

| Type | Type |
| ---------- | ---------- |
| `InstallMethod` | `bun-binary" or "npm" or "pnpm" or "yarn" or "bun" or "unknown` |

### :gear: GitSource

Parsed git URL information.

| Type | Type |
| ---------- | ---------- |
| `GitSource` | `{ /** Always "git" for git sources */ type: "git"; /** Clone URL (always valid for git clone, without ref suffix) */ repo: string; /** Git host domain (e.g., "github.com") */ host: string; /** Repository path (e.g., "user/repo") */ path: string; /** Git ref (branch, tag, commit) if specified */ ref?: string; /** True if ref was specified (package won't be auto-updated) */ pinned: boolean; }` |

### :gear: TransportSetting

| Type | Type |
| ---------- | ---------- |
| `TransportSetting` | `Transport` |

### :gear: PackageSource

Package source for npm/git packages.
- String form: load all resources from the package
- Object form: filter which resources to load

| Type | Type |
| ---------- | ---------- |
| `PackageSource` | `| string or { source: string; extensions?: string[]; skills?: string[]; prompts?: string[]; themes?: string[]; }` |

### :gear: SettingsScope

| Type | Type |
| ---------- | ---------- |
| `SettingsScope` | `global" or "project` |

### :gear: MissingSourceAction

| Type | Type |
| ---------- | ---------- |
| `MissingSourceAction` | `install" or "skip" or "error` |

### :gear: ProgressCallback

| Type | Type |
| ---------- | ---------- |
| `ProgressCallback` | `(event: ProgressEvent) => void` |

### :gear: SourceScope

| Type | Type |
| ---------- | ---------- |
| `SourceScope` | `user" or "project" or "temporary` |

### :gear: SourceOrigin

| Type | Type |
| ---------- | ---------- |
| `SourceOrigin` | `package" or "top-level` |

### :gear: ThemeColor

| Type | Type |
| ---------- | ---------- |
| `ThemeColor` | `| "accent" or "border" or "borderAccent" or "borderMuted" or "success" or "error" or "warning" or "muted" or "dim" or "text" or "thinkingText" or "userMessageText" or "customMessageText" or "customMessageLabel" or "toolTitle" or "toolOutput" or "mdHeading" or "mdLink" or "mdLinkUrl" or "mdCode" or "mdCodeBlock" or "mdCodeBlockBorder" or "mdQuote" or "mdQuoteBorder" or "mdHr" or "mdListBullet" or "toolDiffAdded" or "toolDiffRemoved" or "toolDiffContext" or "syntaxComment" or "syntaxKeyword" or "syntaxFunction" or "syntaxVariable" or "syntaxString" or "syntaxNumber" or "syntaxType" or "syntaxOperator" or "syntaxPunctuation" or "thinkingOff" or "thinkingMinimal" or "thinkingLow" or "thinkingMedium" or "thinkingHigh" or "thinkingXhigh" or "bashMode` |

### :gear: ThemeBg

| Type | Type |
| ---------- | ---------- |
| `ThemeBg` | `| "selectedBg" or "userMessageBg" or "customMessageBg" or "toolPendingBg" or "toolSuccessBg" or "toolErrorBg` |

### :gear: ToolRenderResultLike

| Type | Type |
| ---------- | ---------- |
| `ToolRenderResultLike` | `{ content: (TextContent or ImageContent)[]; details: TDetails; }` |

### :gear: BashToolInput

| Type | Type |
| ---------- | ---------- |
| `BashToolInput` | `Static<typeof bashSchema>` |

### :gear: BashSpawnHook

Spawn 前置钩子，用于在执行前修改命令、工作目录或环境变量

| Type | Type |
| ---------- | ---------- |
| `BashSpawnHook` | `(context: BashSpawnContext) => BashSpawnContext` |

### :gear: SessionEntry

Session entry - has id/parentId for tree structure (returned by "read" methods in SessionManager)

| Type | Type |
| ---------- | ---------- |
| `SessionEntry` | `| SessionMessageEntry or ThinkingLevelChangeEntry or ModelChangeEntry or CompactionEntry or BranchSummaryEntry or CustomEntry or CustomMessageEntry or LabelEntry or SessionInfoEntry` |

### :gear: FileEntry

Raw file entry (includes header)

| Type | Type |
| ---------- | ---------- |
| `FileEntry` | `SessionHeader or SessionEntry` |

### :gear: ReadonlySessionManager

| Type | Type |
| ---------- | ---------- |
| `ReadonlySessionManager` | `Pick< SessionManager, or "getCwd" or "getSessionDir" or "getSessionId" or "getSessionFile" or "getLeafId" or "getLeafEntry" or "getEntry" or "getLabel" or "getBranch" or "getHeader" or "getEntries" or "getTree" or "getSessionName" >` |

### :gear: SessionListProgress

| Type | Type |
| ---------- | ---------- |
| `SessionListProgress` | `(loaded: number, total: number) => void` |

### :gear: ReadonlyFooterDataProvider

Read-only view for extensions - excludes setExtensionStatus, setAvailableProviderCount and dispose

| Type | Type |
| ---------- | ---------- |
| `ReadonlyFooterDataProvider` | `Pick< FooterDataProvider, "getGitBranch" or "getExtensionStatuses" or "getAvailableProviderCount" or "onBranchChange" >` |

### :gear: AppKeybinding

| Type | Type |
| ---------- | ---------- |
| `AppKeybinding` | `keyof AppKeybindings` |

### :gear: ApiKeyCredential

| Type | Type |
| ---------- | ---------- |
| `ApiKeyCredential` | `{ type: "api_key"; key: string; }` |

### :gear: OAuthCredential

| Type | Type |
| ---------- | ---------- |
| `OAuthCredential` | `{ type: "oauth"; } and OAuthCredentials` |

### :gear: AuthCredential

| Type | Type |
| ---------- | ---------- |
| `AuthCredential` | `ApiKeyCredential or OAuthCredential` |

### :gear: AuthStorageData

| Type | Type |
| ---------- | ---------- |
| `AuthStorageData` | `Record<string, AuthCredential>` |

### :gear: AuthStatus

| Type | Type |
| ---------- | ---------- |
| `AuthStatus` | `{ configured: boolean; source?: "stored" or "runtime" or "environment" or "fallback" or "models_json_key" or "models_json_command"; label?: string; }` |

### :gear: ResolvedRequestAuth

| Type | Type |
| ---------- | ---------- |
| `ResolvedRequestAuth` | `| { ok: true; apiKey?: string; headers?: Record<string, string>; } or { ok: false; error: string; }` |

### :gear: SlashCommandSource

| Type | Type |
| ---------- | ---------- |
| `SlashCommandSource` | `extension" or "prompt" or "skill` |

### :gear: EditToolInput

| Type | Type |
| ---------- | ---------- |
| `EditToolInput` | `Static<typeof editSchema>` |

### :gear: FindToolInput

| Type | Type |
| ---------- | ---------- |
| `FindToolInput` | `Static<typeof findSchema>` |

### :gear: GrepToolInput

| Type | Type |
| ---------- | ---------- |
| `GrepToolInput` | `Static<typeof grepSchema>` |

### :gear: LsToolInput

| Type | Type |
| ---------- | ---------- |
| `LsToolInput` | `Static<typeof lsSchema>` |

### :gear: ReadToolInput

| Type | Type |
| ---------- | ---------- |
| `ReadToolInput` | `Static<typeof readSchema>` |

### :gear: WriteToolInput

| Type | Type |
| ---------- | ---------- |
| `WriteToolInput` | `Static<typeof writeSchema>` |

### :gear: Tool

运行时工具实例 —— 对 AgentTool 的泛型封装

| Type | Type |
| ---------- | ---------- |
| `Tool` | `AgentTool<any>` |

### :gear: ToolDef

工具的静态定义（描述、参数 schema 等），用于注册到扩展系统

| Type | Type |
| ---------- | ---------- |
| `ToolDef` | `ToolDefinition<any, any>` |

### :gear: ToolName

所有内置工具名称的字面量联合类型

| Type | Type |
| ---------- | ---------- |
| `ToolName` | `read" or "bash" or "edit" or "write" or "grep" or "find" or "ls` |

### :gear: WidgetPlacement

Placement for extension widgets.

| Type | Type |
| ---------- | ---------- |
| `WidgetPlacement` | `aboveEditor" or "belowEditor` |

### :gear: TerminalInputHandler

Raw terminal input listener for extensions.

| Type | Type |
| ---------- | ---------- |
| `TerminalInputHandler` | `(data: string) => { consume?: boolean; data?: string } or undefined` |

### :gear: AutocompleteProviderFactory

Wrap the current autocomplete provider with additional behavior.

| Type | Type |
| ---------- | ---------- |
| `AutocompleteProviderFactory` | `(current: AutocompleteProvider) => AutocompleteProvider` |

### :gear: EditorFactory

| Type | Type |
| ---------- | ---------- |
| `EditorFactory` | `(tui: TUI, theme: EditorTheme, keybindings: KeybindingsManager) => EditorComponent` |

### :gear: SessionEvent

| Type | Type |
| ---------- | ---------- |
| `SessionEvent` | `| SessionStartEvent or SessionBeforeSwitchEvent or SessionBeforeForkEvent or SessionBeforeCompactEvent or SessionCompactEvent or SessionShutdownEvent or SessionBeforeTreeEvent or SessionTreeEvent` |

### :gear: ModelSelectSource

| Type | Type |
| ---------- | ---------- |
| `ModelSelectSource` | `set" or "cycle" or "restore` |

### :gear: InputSource

Source of user input

| Type | Type |
| ---------- | ---------- |
| `InputSource` | `interactive" or "rpc" or "extension` |

### :gear: InputEventResult

Result from input event handler

| Type | Type |
| ---------- | ---------- |
| `InputEventResult` | `| { action: "continue" } or { action: "transform"; text: string; images?: ImageContent[] } or { action: "handled" }` |

### :gear: ToolCallEvent

Fired before a tool executes. Can block.

`event.input` is mutable. Mutate it in place to patch tool arguments before execution.
Later `tool_call` handlers see earlier mutations. No re-validation is performed after mutation.

| Type | Type |
| ---------- | ---------- |
| `ToolCallEvent` | `| BashToolCallEvent or ReadToolCallEvent or EditToolCallEvent or WriteToolCallEvent or GrepToolCallEvent or FindToolCallEvent or LsToolCallEvent or CustomToolCallEvent` |

### :gear: ToolResultEvent

Fired after a tool executes. Can modify result.

| Type | Type |
| ---------- | ---------- |
| `ToolResultEvent` | `| BashToolResultEvent or ReadToolResultEvent or EditToolResultEvent or WriteToolResultEvent or GrepToolResultEvent or FindToolResultEvent or LsToolResultEvent or CustomToolResultEvent` |

### :gear: ExtensionEvent

Union of all event types

| Type | Type |
| ---------- | ---------- |
| `ExtensionEvent` | `| ResourcesDiscoverEvent or SessionEvent or ContextEvent or BeforeProviderRequestEvent or AfterProviderResponseEvent or BeforeAgentStartEvent or AgentStartEvent or AgentEndEvent or TurnStartEvent or TurnEndEvent or MessageStartEvent or MessageUpdateEvent or MessageEndEvent or ToolExecutionStartEvent or ToolExecutionUpdateEvent or ToolExecutionEndEvent or ModelSelectEvent or ThinkingLevelSelectEvent or UserBashEvent or InputEvent or ToolCallEvent or ToolResultEvent` |

### :gear: BeforeProviderRequestEventResult

| Type | Type |
| ---------- | ---------- |
| `BeforeProviderRequestEventResult` |  |

### :gear: MessageRenderer

| Type | Type |
| ---------- | ---------- |
| `MessageRenderer` | `( message: CustomMessage<T>, options: MessageRenderOptions, theme: Theme, ) => Component or undefined` |

### :gear: ExtensionHandler

Handler function type for events

| Type | Type |
| ---------- | ---------- |
| `ExtensionHandler` | `(event: E, ctx: ExtensionContext) => Promise<R or void> or R or void` |

### :gear: ExtensionFactory

Extension factory function type. Supports both sync and async initialization.

| Type | Type |
| ---------- | ---------- |
| `ExtensionFactory` | `(pi: ExtensionAPI) => void or Promise<void>` |

### :gear: SendMessageHandler

| Type | Type |
| ---------- | ---------- |
| `SendMessageHandler` | `<T = unknown>( message: Pick<CustomMessage<T>, "customType" or "content" or "display" or "details">, options?: { triggerTurn?: boolean; deliverAs?: "steer" or "followUp" or "nextTurn" }, ) => void` |

### :gear: SendUserMessageHandler

| Type | Type |
| ---------- | ---------- |
| `SendUserMessageHandler` | `( content: string or (TextContent or ImageContent)[], options?: { deliverAs?: "steer" or "followUp" }, ) => void` |

### :gear: AppendEntryHandler

| Type | Type |
| ---------- | ---------- |
| `AppendEntryHandler` | `<T = unknown>(customType: string, data?: T) => void` |

### :gear: SetSessionNameHandler

| Type | Type |
| ---------- | ---------- |
| `SetSessionNameHandler` | `(name: string) => void` |

### :gear: GetSessionNameHandler

| Type | Type |
| ---------- | ---------- |
| `GetSessionNameHandler` | `() => string or undefined` |

### :gear: GetActiveToolsHandler

| Type | Type |
| ---------- | ---------- |
| `GetActiveToolsHandler` | `() => string[]` |

### :gear: ToolInfo

Tool info with name, description, parameter schema, and source metadata

| Type | Type |
| ---------- | ---------- |
| `ToolInfo` | `Pick<ToolDefinition, "name" or "description" or "parameters"> and { sourceInfo: SourceInfo; }` |

### :gear: GetAllToolsHandler

| Type | Type |
| ---------- | ---------- |
| `GetAllToolsHandler` | `() => ToolInfo[]` |

### :gear: GetCommandsHandler

| Type | Type |
| ---------- | ---------- |
| `GetCommandsHandler` | `() => SlashCommandInfo[]` |

### :gear: SetActiveToolsHandler

| Type | Type |
| ---------- | ---------- |
| `SetActiveToolsHandler` | `(toolNames: string[]) => void` |

### :gear: RefreshToolsHandler

| Type | Type |
| ---------- | ---------- |
| `RefreshToolsHandler` | `() => void` |

### :gear: SetModelHandler

| Type | Type |
| ---------- | ---------- |
| `SetModelHandler` | `(model: Model<any>) => Promise<boolean>` |

### :gear: GetThinkingLevelHandler

| Type | Type |
| ---------- | ---------- |
| `GetThinkingLevelHandler` | `() => ThinkingLevel` |

### :gear: SetThinkingLevelHandler

| Type | Type |
| ---------- | ---------- |
| `SetThinkingLevelHandler` | `(level: ThinkingLevel) => void` |

### :gear: SetLabelHandler

| Type | Type |
| ---------- | ---------- |
| `SetLabelHandler` | `(entryId: string, label: string or undefined) => void` |

### :gear: Mode

| Type | Type |
| ---------- | ---------- |
| `Mode` | `text" or "json" or "rpc` |

### :gear: SortMode

| Type | Type |
| ---------- | ---------- |
| `SortMode` | `threaded" or "recent" or "relevance` |

### :gear: NameFilter

| Type | Type |
| ---------- | ---------- |
| `NameFilter` | `all" or "named` |

### :gear: ClipboardModule

| Type | Type |
| ---------- | ---------- |
| `ClipboardModule` | `{ setText: (text: string) => Promise<void>; hasImage: () => boolean; getImageBinary: () => Promise<Array<number>>; }` |

### :gear: ClipboardImage

| Type | Type |
| ---------- | ---------- |
| `ClipboardImage` | `{ bytes: Uint8Array; mimeType: string; }` |

### :gear: AuthSelectorProvider

| Type | Type |
| ---------- | ---------- |
| `AuthSelectorProvider` | `{ id: string; name: string; authType: "oauth" or "api_key"; }` |

### :gear: FilterMode

Filter mode for tree display

| Type | Type |
| ---------- | ---------- |
| `FilterMode` | `default" or "no-tools" or "user-only" or "labeled-only" or "all` |

### :gear: RpcCommand

| Type | Type |
| ---------- | ---------- |
| `RpcCommand` | `| { id?: string; type: "prompt"; message: string; images?: ImageContent[]; streamingBehavior?: "steer" or "followUp" } or { id?: string; type: "steer"; message: string; images?: ImageContent[] } or { id?: string; type: "follow_up"; message: string; images?: ImageContent[] } or { id?: string; type: "abort" } or { id?: string; type: "new_session"; parentSession?: string }  // State or { id?: string; type: "get_state" }  // Model or { id?: string; type: "set_model"; provider: string; modelId: string } or { id?: string; type: "cycle_model" } or { id?: string; type: "get_available_models" }  // Thinking or { id?: string; type: "set_thinking_level"; level: ThinkingLevel } or { id?: string; type: "cycle_thinking_level" }  // Queue modes or { id?: string; type: "set_steering_mode"; mode: "all" or "one-at-a-time" } or { id?: string; type: "set_follow_up_mode"; mode: "all" or "one-at-a-time" }  // Compaction or { id?: string; type: "compact"; customInstructions?: string } or { id?: string; type: "set_auto_compaction"; enabled: boolean }  // Retry or { id?: string; type: "set_auto_retry"; enabled: boolean } or { id?: string; type: "abort_retry" }  // Bash or { id?: string; type: "bash"; command: string } or { id?: string; type: "abort_bash" }  // Session or { id?: string; type: "get_session_stats" } or { id?: string; type: "export_html"; outputPath?: string } or { id?: string; type: "switch_session"; sessionPath: string } or { id?: string; type: "fork"; entryId: string } or { id?: string; type: "clone" } or { id?: string; type: "get_fork_messages" } or { id?: string; type: "get_last_assistant_text" } or { id?: string; type: "set_session_name"; name: string }  // Messages or { id?: string; type: "get_messages" }  // Commands (available for invocation via prompt) or { id?: string; type: "get_commands" }` |

### :gear: RpcResponse

| Type | Type |
| ---------- | ---------- |
| `RpcResponse` | `| { id?: string; type: "response"; command: "prompt"; success: true } or { id?: string; type: "response"; command: "steer"; success: true } or { id?: string; type: "response"; command: "follow_up"; success: true } or { id?: string; type: "response"; command: "abort"; success: true } or { id?: string; type: "response"; command: "new_session"; success: true; data: { cancelled: boolean } }  // State or { id?: string; type: "response"; command: "get_state"; success: true; data: RpcSessionState }  // Model or { id?: string; type: "response"; command: "set_model"; success: true; data: Model<any>; } or { id?: string; type: "response"; command: "cycle_model"; success: true; data: { model: Model<any>; thinkingLevel: ThinkingLevel; isScoped: boolean } or null; } or { id?: string; type: "response"; command: "get_available_models"; success: true; data: { models: Model<any>[] }; }  // Thinking or { id?: string; type: "response"; command: "set_thinking_level"; success: true } or { id?: string; type: "response"; command: "cycle_thinking_level"; success: true; data: { level: ThinkingLevel } or null; }  // Queue modes or { id?: string; type: "response"; command: "set_steering_mode"; success: true } or { id?: string; type: "response"; command: "set_follow_up_mode"; success: true }  // Compaction or { id?: string; type: "response"; command: "compact"; success: true; data: CompactionResult } or { id?: string; type: "response"; command: "set_auto_compaction"; success: true }  // Retry or { id?: string; type: "response"; command: "set_auto_retry"; success: true } or { id?: string; type: "response"; command: "abort_retry"; success: true }  // Bash or { id?: string; type: "response"; command: "bash"; success: true; data: BashResult } or { id?: string; type: "response"; command: "abort_bash"; success: true }  // Session or { id?: string; type: "response"; command: "get_session_stats"; success: true; data: SessionStats } or { id?: string; type: "response"; command: "export_html"; success: true; data: { path: string } } or { id?: string; type: "response"; command: "switch_session"; success: true; data: { cancelled: boolean } } or { id?: string; type: "response"; command: "fork"; success: true; data: { text: string; cancelled: boolean } } or { id?: string; type: "response"; command: "clone"; success: true; data: { cancelled: boolean } } or { id?: string; type: "response"; command: "get_fork_messages"; success: true; data: { messages: Array<{ entryId: string; text: string }> }; } or { id?: string; type: "response"; command: "get_last_assistant_text"; success: true; data: { text: string or null }; } or { id?: string; type: "response"; command: "set_session_name"; success: true }  // Messages or { id?: string; type: "response"; command: "get_messages"; success: true; data: { messages: AgentMessage[] } }  // Commands or { id?: string; type: "response"; command: "get_commands"; success: true; data: { commands: RpcSlashCommand[] }; }  // Error response (any command can fail) or { id?: string; type: "response"; command: string; success: false; error: string }` |

### :gear: RpcExtensionUIRequest

Emitted when an extension needs user input

| Type | Type |
| ---------- | ---------- |
| `RpcExtensionUIRequest` | `| { type: "extension_ui_request"; id: string; method: "select"; title: string; options: string[]; timeout?: number } or { type: "extension_ui_request"; id: string; method: "confirm"; title: string; message: string; timeout?: number } or { type: "extension_ui_request"; id: string; method: "input"; title: string; placeholder?: string; timeout?: number; } or { type: "extension_ui_request"; id: string; method: "editor"; title: string; prefill?: string } or { type: "extension_ui_request"; id: string; method: "notify"; message: string; notifyType?: "info" or "warning" or "error"; } or { type: "extension_ui_request"; id: string; method: "setStatus"; statusKey: string; statusText: string or undefined; } or { type: "extension_ui_request"; id: string; method: "setWidget"; widgetKey: string; widgetLines: string[] or undefined; widgetPlacement?: "aboveEditor" or "belowEditor"; } or { type: "extension_ui_request"; id: string; method: "setTitle"; title: string } or { type: "extension_ui_request"; id: string; method: "set_editor_text"; text: string }` |

### :gear: RpcExtensionUIResponse

Response to an extension UI request

| Type | Type |
| ---------- | ---------- |
| `RpcExtensionUIResponse` | `| { type: "extension_ui_response"; id: string; value: string } or { type: "extension_ui_response"; id: string; confirmed: boolean } or { type: "extension_ui_response"; id: string; cancelled: true }` |

### :gear: RpcCommandType

| Type | Type |
| ---------- | ---------- |
| `RpcCommandType` | `RpcCommand["type"]` |

### :gear: RpcEventListener

| Type | Type |
| ---------- | ---------- |
| `RpcEventListener` | `(event: AgentEvent) => void` |

### :gear: ExtensionErrorListener

| Type | Type |
| ---------- | ---------- |
| `ExtensionErrorListener` | `(error: ExtensionError) => void` |

### :gear: NewSessionHandler

| Type | Type |
| ---------- | ---------- |
| `NewSessionHandler` | `(options?: { parentSession?: string; setup?: (sessionManager: SessionManager) => Promise<void>; withSession?: (ctx: ReplacedSessionContext) => Promise<void>; }) => Promise<{ cancelled: boolean }>` |

### :gear: ForkHandler

| Type | Type |
| ---------- | ---------- |
| `ForkHandler` | `( entryId: string, options?: { position?: "before" or "at"; withSession?: (ctx: ReplacedSessionContext) => Promise<void> }, ) => Promise<{ cancelled: boolean }>` |

### :gear: NavigateTreeHandler

| Type | Type |
| ---------- | ---------- |
| `NavigateTreeHandler` | `( targetId: string, options?: { summarize?: boolean; customInstructions?: string; replaceInstructions?: boolean; label?: string }, ) => Promise<{ cancelled: boolean }>` |

### :gear: SwitchSessionHandler

| Type | Type |
| ---------- | ---------- |
| `SwitchSessionHandler` | `( sessionPath: string, options?: { withSession?: (ctx: ReplacedSessionContext) => Promise<void> }, ) => Promise<{ cancelled: boolean }>` |

### :gear: ReloadHandler

| Type | Type |
| ---------- | ---------- |
| `ReloadHandler` | `() => Promise<void>` |

### :gear: ShutdownHandler

| Type | Type |
| ---------- | ---------- |
| `ShutdownHandler` | `() => void` |

### :gear: AgentSessionEvent

会话级别的扩展事件，在核心 AgentEvent 基础上增加队列状态、压缩进度、自动重试等事件

| Type | Type |
| ---------- | ---------- |
| `AgentSessionEvent` | `| AgentEvent or { type: "queue_update"; steering: readonly string[]; followUp: readonly string[]; } or { type: "compaction_start"; reason: "manual" or "threshold" or "overflow" } or { type: "session_info_changed"; name: string or undefined } or { type: "thinking_level_changed"; level: ThinkingLevel } or { type: "compaction_end"; reason: "manual" or "threshold" or "overflow"; result: CompactionResult or undefined; aborted: boolean; willRetry: boolean; errorMessage?: string; } or { type: "auto_retry_start"; attempt: number; maxAttempts: number; delayMs: number; errorMessage: string } or { type: "auto_retry_end"; success: boolean; attempt: number; finalError?: string }` |

### :gear: AgentSessionEventListener

Agent 会话事件监听器函数类型

| Type | Type |
| ---------- | ---------- |
| `AgentSessionEventListener` | `(event: AgentSessionEvent) => void` |

### :gear: CreateAgentSessionRuntimeFactory

Creates a full runtime for a target cwd and session manager.

The factory closes over process-global fixed inputs, recreates cwd-bound
services for the effective cwd, resolves session options against those
services, and finally creates the AgentSession.

| Type | Type |
| ---------- | ---------- |
| `CreateAgentSessionRuntimeFactory` | `(options: { cwd: string; agentDir: string; sessionManager: SessionManager; sessionStartEvent?: SessionStartEvent; }) => Promise<CreateAgentSessionRuntimeResult>` |

### :gear: PackageCommand

| Type | Type |
| ---------- | ---------- |
| `PackageCommand` | `install" or "remove" or "update" or "list` |

