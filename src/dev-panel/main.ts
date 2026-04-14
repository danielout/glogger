import { createApp } from "vue";
import { createPinia } from "pinia";
import "../assets/css/main.css";
import DevPanel from "./DevPanel.vue";

const app = createApp(DevPanel);
app.use(createPinia());
app.mount("#app");
