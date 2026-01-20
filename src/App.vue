<template>
  <div class="container">
    <div class="header">
      <h1>🤖 RoboRIO File Explorer</h1>
      <div class="mode-selector">
        <button 
          :class="['mode-btn', { active: connectionMode === 'phoenix' }]"
          @click="connectionMode = 'phoenix'"
        >
          Phoenix Tuner X
        </button>
        <button 
          :class="['mode-btn', { active: connectionMode === 'ssh' }]"
          @click="connectionMode = 'ssh'"
        >
          SSH/SFTP
        </button>
      </div>
      <div class="connection-form">
        <div v-if="connectionMode === 'phoenix'" class="phoenix-form">
          <div class="form-group">
            <label>RoboRIO Host</label>
            <input 
              v-model="host" 
              placeholder="roboRIO-TEAM.local or 10.xx.xx.2"
              @keyup.enter="connectPhoenix"
            />
          </div>
          <button @click="connectPhoenix" :disabled="!host || loading">
            {{ loading ? 'Connecting...' : 'Connect' }}
          </button>
        </div>
        <div v-else class="ssh-form">
          <div class="form-group">
            <label>RoboRIO Host</label>
            <input 
              v-model="host" 
              placeholder="roboRIO-TEAM.local or 10.xx.xx.2"
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
              placeholder="password"
              @keyup.enter="connectSSH"
            />
          </div>
          <button @click="connectSSH" :disabled="!host || !sshUsername || loading">
            {{ loading ? 'Connecting...' : 'Connect' }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="statusMessage" :class="['status', statusType]">
      {{ statusMessage }}
    </div>

    <div v-if="connected" class="toolbar">
      <input 
        v-model="currentPath" 
        type="text" 
        class="path-input"
        placeholder="/root"
        @keyup.enter="listFiles"
      />
      <button @click="listFiles" :disabled="loading">{{ loading ? 'Loading...' : 'Refresh' }}</button>
      <button @click="goBack" :disabled="loading">Back</button>
      <button @click="uploadFile" :disabled="loading">Upload</button>
      <input 
        ref="fileInput" 
        type="file" 
        style="display: none" 
        @change="handleFileSelect"
      />
    </div>

    <div v-if="connected" class="content">
      <div class="file-list">
        <div v-if="errorMessage" class="error-message">
          {{ errorMessage }}
        </div>
        <div v-if="successMessage" class="success-message">
          {{ successMessage }}
        </div>

        <div v-if="files.length === 0 && !loading" class="empty">
          <p>No files found or not connected</p>
        </div>

        <div v-for="file in files" :key="file.path" class="file-item">
          <div class="file-icon">
            {{ file.is_dir ? '📁' : '📄' }}
          </div>
          <div class="file-info" @click="openFile(file)">
            <div class="file-name">{{ file.name }}</div>
            <div class="file-meta">
              {{ file.is_dir ? 'Folder' : formatSize(file.size) }} · 
              {{ formatDate(file.modified) }}
            </div>
          </div>
          <div class="file-actions">
            <button v-if="!file.is_dir" @click="downloadFile(file)" :disabled="loading">
              Download
            </button>
            <button @click="deleteFile(file)" :disabled="loading">
              Delete
            </button>
          </div>
        </div>
      </div>
    </div>

    <div v-else class="empty" style="display: flex; align-items: center; justify-content: center;">
      <p>Connect to your RoboRIO to get started</p>
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

    return {
      connectionMode,
      host,
      sshUsername,
      sshPassword,
      currentPath,
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
    }
  },
}
</script>
