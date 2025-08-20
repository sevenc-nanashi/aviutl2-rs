import { createApp } from "vue";
import "./style.css";
import App from "./App.vue";
import { renderDataPlugin } from "./useRenderData";

createApp(App).use(renderDataPlugin).mount("#app");
