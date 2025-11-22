package io.bazbom.intellij.quickfix

import com.intellij.codeInsight.intention.IntentionAction
import com.intellij.codeInsight.intention.PriorityAction
import com.intellij.openapi.command.WriteCommandAction
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.editor.Editor
import com.intellij.openapi.progress.ProgressIndicator
import com.intellij.openapi.progress.ProgressManager
import com.intellij.openapi.progress.Task
import com.intellij.openapi.project.Project
import com.intellij.psi.PsiFile
import com.intellij.psi.xml.XmlTag
import com.intellij.notification.NotificationGroupManager
import com.intellij.notification.NotificationType
import org.jetbrains.idea.maven.project.MavenProjectsManager

/**
 * Quick fix to upgrade a vulnerable dependency to a safe version.
 * Appears as Alt+Enter option on highlighted dependencies.
 */
class UpgradeDependencyQuickFix(
    private val groupId: String,
    private val artifactId: String,
    private val currentVersion: String,
    private val targetVersion: String
) : IntentionAction, PriorityAction {
    
    private val log = Logger.getInstance(UpgradeDependencyQuickFix::class.java)
    
    override fun getText(): String = "Upgrade $artifactId to safe version $targetVersion"
    
    override fun getFamilyName(): String = "BazBOM Quick Fixes"
    
    override fun isAvailable(project: Project, editor: Editor?, file: PsiFile?): Boolean = true
    
    override fun invoke(project: Project, editor: Editor?, file: PsiFile?) {
        if (file == null) return
        
        WriteCommandAction.runWriteCommandAction(project) {
            try {
                // Find the dependency tag
                val dependencyTag = findDependencyTag(file, groupId, artifactId)
                if (dependencyTag != null) {
                    // Update version
                    val versionTag = dependencyTag.findFirstSubTag("version")
                    if (versionTag != null) {
                        versionTag.value.text = targetVersion
                        
                        log.info("Updated $artifactId from $currentVersion to $targetVersion")
                        
                        // Reload Maven project
                        reloadMavenProject(project)
                        
                        // Run tests in background
                        runTestsInBackground(project)
                    }
                }
            } catch (e: Exception) {
                log.error("Failed to upgrade dependency", e)
                showNotification(
                    project,
                    "Upgrade Failed",
                    "Failed to upgrade $artifactId: ${e.message}",
                    NotificationType.ERROR
                )
            }
        }
    }
    
    private fun reloadMavenProject(project: Project) {
        val mavenProjectsManager = MavenProjectsManager.getInstance(project)
        mavenProjectsManager.forceUpdateAllProjectsOrFindAllAvailablePomFiles()
    }
    
    private fun runTestsInBackground(project: Project) {
        ProgressManager.getInstance().run(object : Task.Backgroundable(
            project,
            "Running tests after upgrading $artifactId to $targetVersion",
            true
        ) {
            override fun run(indicator: ProgressIndicator) {
                indicator.text = "Running Maven tests..."
                indicator.isIndeterminate = true
                
                try {
                    // Use ProcessBuilder to run Maven tests
                    val processBuilder = ProcessBuilder("mvn", "test", "-DskipTests=false", "--batch-mode")
                    processBuilder.directory(java.io.File(project.basePath ?: "."))
                    processBuilder.redirectErrorStream(true)
                    
                    val process = processBuilder.start()
                    val exitCode = process.waitFor()
                    
                    if (exitCode == 0) {
                        showNotification(
                            project,
                            "Upgrade Successful",
                            "Upgraded $artifactId to $targetVersion. All tests passed.",
                            NotificationType.INFORMATION
                        )
                    } else {
                        showNotification(
                            project,
                            "Tests Failed",
                            "Upgraded $artifactId to $targetVersion but tests failed. Please review and fix.",
                            NotificationType.WARNING
                        )
                    }
                } catch (e: Exception) {
                    log.warn("Failed to run tests", e)
                    showNotification(
                        project,
                        "Upgrade Applied",
                        "Upgraded $artifactId to $targetVersion. Run tests manually to verify.",
                        NotificationType.INFORMATION
                    )
                }
            }
        })
    }
    
    private fun showNotification(project: Project, title: String, content: String, type: NotificationType) {
        NotificationGroupManager.getInstance()
            .getNotificationGroup("BazBOM.Notifications")
            .createNotification(title, content, type)
            .notify(project)
    }
    
    override fun startInWriteAction(): Boolean = false
    
    override fun getPriority(): PriorityAction.Priority = PriorityAction.Priority.HIGH
    
    private fun findDependencyTag(file: PsiFile, groupId: String, artifactId: String): XmlTag? {
        // Simple search through XML tags
        // In production, would use more robust PSI navigation
        val allTags = mutableListOf<XmlTag>()
        collectXmlTags(file, allTags)
        
        return allTags.firstOrNull { tag ->
            if (tag.name != "dependency") return@firstOrNull false
            
            val tagGroupId = tag.findFirstSubTag("groupId")?.value?.text
            val tagArtifactId = tag.findFirstSubTag("artifactId")?.value?.text
            
            tagGroupId == groupId && tagArtifactId == artifactId
        }
    }
    
    private fun collectXmlTags(element: PsiFile, result: MutableList<XmlTag>) {
        for (child in element.children) {
            if (child is XmlTag) {
                result.add(child)
                collectXmlTags(child as PsiFile, result)
            }
        }
    }
}
