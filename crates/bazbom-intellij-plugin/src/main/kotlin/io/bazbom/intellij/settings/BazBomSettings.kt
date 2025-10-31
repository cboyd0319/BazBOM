package io.bazbom.intellij.settings

import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.components.PersistentStateComponent
import com.intellij.openapi.components.State
import com.intellij.openapi.components.Storage
import com.intellij.util.xmlb.XmlSerializerUtil

/**
 * Persistent settings for BazBOM plugin.
 */
@State(
    name = "BazBomSettings",
    storages = [Storage("bazbom.xml")]
)
class BazBomSettings : PersistentStateComponent<BazBomSettings> {
    
    var enableRealTimeScanning: Boolean = true
    var showInlineWarnings: Boolean = true
    var autoScanOnSave: Boolean = true
    var autoScanOnOpen: Boolean = false
    var showCritical: Boolean = true
    var showHigh: Boolean = true
    var showMedium: Boolean = true
    var showLow: Boolean = false
    var policyFilePath: String = "bazbom.yml"
    var cliPath: String = "/usr/local/bin/bazbom"
    
    override fun getState(): BazBomSettings {
        return this
    }
    
    override fun loadState(state: BazBomSettings) {
        XmlSerializerUtil.copyBean(state, this)
    }
    
    companion object {
        fun getInstance(): BazBomSettings {
            return ApplicationManager.getApplication().getService(BazBomSettings::class.java)
        }
    }
}
