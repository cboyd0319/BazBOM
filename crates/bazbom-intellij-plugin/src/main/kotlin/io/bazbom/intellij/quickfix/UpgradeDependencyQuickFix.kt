package io.bazbom.intellij.quickfix

import com.intellij.codeInsight.intention.IntentionAction
import com.intellij.codeInsight.intention.PriorityAction
import com.intellij.openapi.command.WriteCommandAction
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.editor.Editor
import com.intellij.openapi.project.Project
import com.intellij.psi.PsiFile
import com.intellij.psi.xml.XmlTag

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
                        val newVersionTag = versionTag.createChildTag(
                            "version",
                            null,
                            targetVersion,
                            false
                        )
                        versionTag.value.text = targetVersion
                        
                        log.info("Updated $artifactId from $currentVersion to $targetVersion")
                        
                        // TODO: Run Maven reload
                        // TODO: Run tests in background
                        // TODO: Show notification with results
                    }
                }
            } catch (e: Exception) {
                log.error("Failed to upgrade dependency", e)
            }
        }
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
