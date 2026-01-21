<template>
  <div class="container">
    <div class="sidebar">
      <div class="sidebar-header">
        <h1 class="app-title">RoboRIO</h1>
      </div>
      <div class="sidebar-section">
        <h3 class="sidebar-section-title">Navigation</h3>
        <div class="sidebar-item active">
          <span class="sidebar-icon">📁</span>
          <span>Files</span>
        </div>
        <div class="sidebar-item">
          <span class="sidebar-icon">⭐</span>
          <span>Favorites</span>
        </div>
        <div class="sidebar-item">
          <span class="sidebar-icon">📤</span>
          <span>Uploads</span>
        </div>
      </div>
      <div class="sidebar-section">
        <h3 class="sidebar-section-title">Quick Tags</h3>
        <div class="tag-item" style="--tag-color: #ef4444;">
          <span class="tag-dot"></span>
          <span>Critical</span>
        </div>
        <div class="tag-item" style="--tag-color: #f97316;">
          <span class="tag-dot"></span>
          <span>Important</span>
        </div>
        <div class="tag-item" style="--tag-color: #eab308;">
          <span class="tag-dot"></span>
          <span>Active</span>
        </div>
      </div>
    </div>

    <div class="main-content">
      <div class="header">
        <div class="header-left">
          <button class="nav-btn" @click="goBack" title="Go back">‹</button>
          <button class="nav-btn" title="Go forward">›</button>
          <div class="breadcrumb">
            <span>RoboRIO</span>
            <span v-for="part in currentPathParts" :key="part" class="breadcrumb-part">/ {{ part }}</span>
          </div>
        </div>
        <div class="header-right">
          <div class="mode-selector">
            <button 
              :class="['mode-btn', { active: connectionMode === 'phoenix' }]"
              @click="connectionMode = 'phoenix'"
              title="Phoenix Tuner X"
            >
              🌐
            </button>
            <button 
              :class="['mode-btn', { active: connectionMode === 'ssh' }]"
              @click="connectionMode = 'ssh'"
              title="SSH/SFTP"
            >
              🔐
            </button>
          </div>
        </div>
      </div>

      <div class="connection-panel">
        <div v-if="connectionMode === 'phoenix'" class="phoenix-form">
          <div class="form-group">
            <label>RoboRIO Host</label>
            <input 
              v-model="host" 
              placeholder="roboRIO-TEAM.local"
              @keyup.enter="connectPhoenix"
            />
          </div>
          <button @click="connectPhoenix" :disabled="!host || loading" class="connect-btn">
            {{ loading ? '⏳ Connecting...' : '✨ Connect' }}
          </button>
        </div>
        <div v-else class="ssh-form">
          <div class="form-group">
            <label>RoboRIO Host</label>
            <input 
              v-model="host" 
              placeholder="roboRIO-TEAM.local"
              @keyup.enter="connectSSH"
            />
          </div>
          <div class="form-group">
            <label>Username</label>
            <input 
              v-model="sshUsername" 
              placeholder="lvuser"
              @keyup.enter="connectSSH"
            />
          </div>
          <div class="form-group">
            <label>Password</label>
            <input 
              v-model="sshPassword" 
              type="password"
              placeholder="••••••••"
              @keyup.enter="connectSSH"
            />
          </div>
          <button @click="connectSSH" :disabled="!host || !sshUsername || loading" class="connect-btn">
            {{ loading ? '⏳ Connecting...' : '✨ Connect' }}
          </button>
        </div>
      </div>

      <div v-if="statusMessage" :class="['status-banner', statusType]">
        {{ statusMessage }}
      </div>

      <div v-if="connected" class="files-section">
        <div v-if="errorMessage" class="alert alert-error">
          ⚠️ {{ errorMessage }}
        </div>
        <div v-if="successMessage" class="alert alert-success">
          ✓ {{ successMessage }}
        </div>

        <div v-if="files.length === 0 && !loading" class="empty-state">
          <p>📭 No files found</p>
        </div>

        <div class="files-grid">
          <div v-for="file in files" :key="file.path" class="file-card">
            <div class="file-thumbnail">
              {{ file.is_dir ? '📁' : getFileIcon(file.name) }}
            </div>
            <div class="file-details">
              <div class="file-name">{{ file.name }}</div>
              <div class="file-size">{{ file.is_dir ? 'Folder' : formatSize(file.size) }}</div>
            </div>
            <div class="file-actions">
              <button 
                v-if="!file.is_dir" 
                @click="downloadFile(file)" 
                :disabled="loading" 
                class="action-btn"
                title="Download"
              >
                ⬇️
              </button>
              <button 
                @click="openFile(file)" 
                :disabled="loading" 
                class="action-btn"
                :title="file.is_dir ? 'Open folder' : 'Open file'"
              >
                ▶️
              </button>
              <button 
                @click="deleteFile(file)" 
                :disabled="loading" 
                class="action-btn danger"
                title="Delete"
              >
                🗑️
              </button>
            </div>
          </div>
        </div>

        <div class="toolbar">
          <button @click="uploadFile" :disabled="loading" class="toolbar-btn">
            ⬆️ Upload
          </button>
          <button @click="listFiles" :disabled="loading" class="toolbar-btn">
            🔄 Refresh
          </button>
          <input 
            ref="fileInput" 
            type="file" 
            style="display: none" 
            @change="handleFileSelect"
          />
        </div>
      </div>

      <div v-else class="empty-state-full">
        <div class="empty-content">
          <p class="empty-title">🚀 Ready to Connect</p>
          <p class="empty-subtitle">Select a connection mode and enter your RoboRIO details</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

export default {
  name: 'App',
  setup() {
    const connectionMode = ref('phoenix')
    const host = ref('roboRIO-TEAM.local')
    const sshUsername = ref('lvuser')
    const sshPassword = ref('')
    const currentPath = ref('/root')
    const files = ref([])
    const connected = ref(false)
    const loading = ref(false)
    const statusMessage = ref('')
    const statusType = ref('')
    const errorMessage = ref('')
    const successMessage = ref('')
    const fileInput = ref(null)

    const connectPhoenix = async () => {
      loading.value = true
      statusMessage.value = ''
      errorMessage.value = ''
      
      try {
        const result = await invoke('connect_phoenix', {
          host: host.value,
        })
        connected.value = true
        statusMessage.value = result
        statusType.value = 'connected'
        await listFiles()
      } catch (error) {
        connected.value = false
        statusMessage.value = `✗ Connection failed`
        statusType.value = 'error'
        errorMessage.value = error
      } finally {
        loading.value = false
      }
    }

    const connectSSH = async () => {
      loading.value = true
      statusMessage.value = ''
      errorMessage.value = ''
      
      try {
        const result = await invoke('connect_ssh', {
          host: host.value,
          username: sshUsername.value,
          password: sshPassword.value,
        })
        connected.value = true
        statusMessage.value = result
        statusType.value = 'connected'
        currentPath.value = '/home/lvuser'
        await listFiles()
      } catch (error) {
        connected.value = false
        statusMessage.value = `✗ Connection failed`
        statusType.value = 'error'
        errorMessage.value = error
      } finally {
        loading.value = false
      }
    }

    const listFiles = async () => {
      loading.value = true
      errorMessage.value = ''
      
      try {
        const response = await invoke('list_files', {
          path: currentPath.value,
        })
        files.value = response.sort((a, b) => {
          if (a.is_dir !== b.is_dir) return b.is_dir ? 1 : -1
          return a.name.localeCompare(b.name)
        })
      } catch (error) {
        errorMessage.value = `Failed to list files: ${error}`
        files.value = []
      } finally {
        loading.value = false
      }
    }

    const openFile = (file) => {
      if (file.is_dir) {
        currentPath.value = file.path
        listFiles()
      }
    }

    const goBack = () => {
      const parts = currentPath.value.split('/')
      parts.pop()
      currentPath.value = parts.join('/') || '/'
      listFiles()
    }

    const downloadFile = async (file) => {
      loading.value = true
      errorMessage.value = ''
      successMessage.value = ''
      
      try {
        const { save } = await import('@tauri-apps/api/dialog')
        const filePath = await save({
          defaultPath: file.name,
          filters: [{ name: 'All', extensions: ['*'] }],
        })
        
        if (filePath) {
          await invoke('download_file', {
            remotePath: file.path,
            localPath: filePath,
          })
          successMessage.value = `Downloaded ${file.name}`
          setTimeout(() => { successMessage.value = '' }, 3000)
        }
      } catch (error) {
        errorMessage.value = `Failed to download: ${error}`
      } finally {
        loading.value = false
      }
    }

    const deleteFile = async (file) => {
      if (!confirm(`Delete ${file.name}?`)) return
      
      loading.value = true
      errorMessage.value = ''
      successMessage.value = ''
      
      try {
        await invoke('delete_file', {
          path: file.path,
        })
        successMessage.value = `Deleted ${file.name}`
        await listFiles()
        setTimeout(() => { successMessage.value = '' }, 3000)
      } catch (error) {
        errorMessage.value = `Failed to delete: ${error}`
      } finally {
        loading.value = false
      }
    }

    const uploadFile = () => {
      fileInput.value.click()
    }

    const handleFileSelect = async (event) => {
      errorMessage.value = 'File upload requires additional setup. Please use SFTP client or implement proper file handling.'
      event.target.value = ''
    }

    const formatSize = (bytes) => {
      if (bytes === 0) return '0 B'
      const k = 1024
      const sizes = ['B', 'KB', 'MB', 'GB']
      const i = Math.floor(Math.log(bytes) / Math.log(k))
      return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i]
    }

    const formatDate = (timestamp) => {
      return new Date(timestamp * 1000).toLocaleDateString()
    }

    const getFileIcon = (filename) => {
      const ext = filename.split('.').pop().toLowerCase()
      const icons = {
        'pdf': '📕', 'doc': '📘', 'docx': '📘', 'txt': '📄',
        'jpg': '🖼️', 'jpeg': '🖼️', 'png': '🖼️', 'gif': '🖼️',
        'mp3': '🎵', 'mp4': '🎬', 'avi': '🎬',
        'zip': '📦', 'rar': '📦', '7z': '📦',
        'exe': '⚙️', 'sh': '⚙️', 'py': '🐍', 'js': '📜',
      }
      return icons[ext] || '📄'
    }

    const currentPathParts = ref([])
    
    const updatePathParts = () => {
      currentPathParts.value = currentPath.value.split('/').filter(p => p)
    }

    return {
      connectionMode,
      host,
      sshUsername,
      sshPassword,
      currentPath,
      currentPathParts,
      files,
      connected,
      loading,
      statusMessage,
      statusType,
      errorMessage,
      successMessage,
      fileInput,
      connectPhoenix,
      connectSSH,
      listFiles,
      openFile,
      goBack,
      downloadFile,
      deleteFile,
      uploadFile,
      handleFileSelect,
      formatSize,
      formatDate,
      getFileIcon,
    }
  },
}
</script>
