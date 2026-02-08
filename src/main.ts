import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";

try {
    const pinia = createPinia();
    const app = createApp(App);
    app.use(pinia);
    app.mount("#app");
} catch (e) {
    document.title = `Erreur démarrage: ${e}`;
}
