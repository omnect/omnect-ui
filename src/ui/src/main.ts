import { createApp } from "vue"
import "./style.css"
import App from "./App.vue"
import { registerPlugins } from "./plugins"
import "virtual:uno.css"

// Import test helpers for browser console (registers window.coreTest)
import "./test-core"

const app = createApp(App)

registerPlugins(app)

app.mount("#app")
