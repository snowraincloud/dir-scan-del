<template>
  <div class="p-10 bg-white min-h-full rounded">
    <!-- <el-card class="content-query">

    </el-card> -->
    <div class="flex-end">
      <!-- <el-button @click="reset">重置</el-button> -->
      <el-button type="primary" @click="openScan">扫描</el-button>
      <el-button type="danger" @click="del">删除</el-button>
    </div>
    <div class="content">
      <h2>总大小: {{ allSize }}</h2>
      <el-table :data="tableDate" stripe :border="true" style="width: 100%" highlight-current-row     @selection-change="handleSelectionChange"
>
        <el-table-column type="selection" width="55" />
        <template v-for="(itemProps, index) in tableColProps" :key="index">

          <el-table-column v-bind="itemProps" align="center" :width="itemProps.width">
            <template #default="scope">

              <template v-if="!itemProps.type">
                {{
        scope.row[itemProps.prop], itemProps.prop
      }}
              </template>

            </template>
          </el-table-column>
        </template>
      </el-table>
    </div>
    <el-dialog title="扫描文件夹" v-model="scanVisiable" width="500px">
      <el-form :model="scanForm" label-width="120px">
        <el-form-item label="文件夹">
          <el-input v-model="scanForm.path">
            <template #suffix>
              <div @click="getScanDir" class="cursor-pointer">
                <el-icon>
                  <FolderOpened />
                </el-icon>
              </div>
            </template>
          </el-input>
        </el-form-item>
        <el-form-item label="目标文件夹名称">
          <div class="flex gap-2">
            <el-tag v-for="tag in dynamicTags" :key="tag" closable :disable-transitions="false"
              @close="handleClose(tag)">
              {{ tag }}
            </el-tag>
            <el-input v-if="inputVisible" ref="InputRef" v-model="inputValue" class="w-20" size="small"
              @keyup.enter="handleInputConfirm" @blur="handleInputConfirm" />
            <el-button v-else class="button-new-tag" size="small" @click="showInput">
              +
            </el-button>
          </div>
        </el-form-item>
      </el-form>
      <div slot="footer" class="flex flex-end">
        <el-button @click="scanVisiable = false">取 消</el-button>
        <el-button type="primary" @click="scan">确 定</el-button>
      </div>
    </el-dialog>
  </div>
</template>
<script setup>
import { ref, nextTick } from 'vue';
import { open } from "@tauri-apps/api/dialog";
import { FolderOpened } from '@element-plus/icons-vue';
import { invoke } from '@tauri-apps/api/tauri';
import { ElMessage } from 'element-plus';
const scanVisiable = ref(false)

const scanForm = ref({
  path: '',
  target: []
})


const inputValue = ref('')
const dynamicTags = ref(['target', 'node_modules'])
const inputVisible = ref(false)
const InputRef = ref()

const handleClose = (tag) => {
  dynamicTags.value.splice(dynamicTags.value.indexOf(tag), 1)
}

const showInput = () => {
  inputVisible.value = true
  nextTick(() => {
    InputRef.value.input.focus()
  })
}

const handleInputConfirm = () => {
  if (inputValue.value) {
    dynamicTags.value.push(inputValue.value)
  }
  inputVisible.value = false
  inputValue.value = ''
}

const getScanDir = async () => {
  let file = await open({ directory: true, multiple: false, title: "请选择要扫描的文件夹" })
  scanForm.value = {
    ...scanForm.value,
    path: file
  }
}

const openScan = () => {
  scanVisiable.value = true
}


const scanTableColProps = [
  { prop: 'path', label: '文件路径', width: 'auto' },
  { prop: 'size', label: '大小', width: '100'},
  { prop: 'status', label: '状态', width: 'auto'},
]

const scan = () => {
  console.log(scanForm.value)
  console.log(dynamicTags.value)
  // check
  if (!scanForm.value.path) {
    ElMessage.error('请选择文件夹')
    return
  }
  if (dynamicTags.value.length === 0) {
    ElMessage.error('请填写目标文件夹名称')
    return
  }
  invoke('scan', { directory: scanForm.value.path, targetDirectory: dynamicTags.value }).then((res) => {
    console.log(res)
    tableColProps.value = scanTableColProps
    tableDate.value = res[0].map(item => {
      let size = item[1]
      if (size > 1024 * 1024 * 1024){
        size = (size / 1024 / 1024 / 1024).toFixed(2) + 'G'
      } else if (size > 1024 * 1024){
        size = (size / 1024 / 1024).toFixed(2) + 'M'
      } else if (size > 1024){
        size = (size / 1024).toFixed(2) + 'K'
      } else {
        size = size + 'B'
      }
      return {
        path: item[0],
        size: size,
        rawSize: item[1],
        status: item[2]
      }
    })
    allSize.value = res[1]
    allSize.value = allSize.value > 1024 * 1024 * 1024 ? (allSize.value / 1024 / 1024 / 1024).toFixed(2) + 'G' : (allSize.value / 1024 / 1024).toFixed(2) + 'M'
    ElMessage.success('扫描成功')
    scanVisiable.value = false
  }).catch(err => {
    console.error(err)
    ElMessage.error('扫描失败')
  })
}

const delDirs = ref([])

const handleSelectionChange = (val) => {
  delDirs.value = val
}
const delTableColProps = [
  { prop: 'path', label: '文件路径', width: 'auto' },
  { prop: 'status', label: '状态', width: 'auto'},
]
const del = () => {
  if (delDirs.value.length === 0) {
    ElMessage.error('请选择要删除的文件')
    return
  }
  invoke('del', { targetDirectory: delDirs.value.map(item => item.path) }).then((res) => {
    console.log(res)
    tableColProps.value = delTableColProps
    tableDate.value = res.map(item => {
      return {
        path: item[0],
        status: item[1]
      }
    })
    allSize.value = delDirs.value.reduce((pre, cur) => {
      return pre + cur.rawSize
    }, 0)
    allSize.value = allSize.value > 1024 * 1024 * 1024 ? (allSize.value / 1024 / 1024 / 1024).toFixed(2) + 'G' : (allSize.value / 1024 / 1024).toFixed(2) + 'M'
    ElMessage.success('删除成功')
  }).catch(err => {
    console.error(err)
    ElMessage.error('删除失败')
  })
}

const allSize = ref(0)
const tableColProps = ref(scanTableColProps)

const tableDate = ref([

])




</script>
<style scoped>
.content-query {
  margin-bottom: 20px;
  display: flex;

}

.flex-end {
  display: flex;
  justify-content: flex-end;
}

.content {
  margin-top: 20px;
  padding: 10;
}
</style>