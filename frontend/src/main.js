import { createApp } from 'vue'
import App from './App.vue'
import router from './router/index.js'
import './styles/variables.css'
import './styles/layout.css'
import './styles/graph.css'

const app = createApp(App)
app.use(router)
app.mount('#app')
