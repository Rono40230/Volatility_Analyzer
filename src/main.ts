import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";

// DEBUG: Catch initialization errors
window.addEventListener('error', (event) => {
    alert(`Global Error: ${event.message}\n${event.filename}:${event.lineno}`);
});

window.addEventListener('unhandledrejection', (event) => {
    alert(`Unhandled Rejection: ${event.reason}`);
});

try {
    const pinia = createPinia();
    const app = createApp(App);

    app.config.errorHandler = (err, instance, info) => {
        alert(`Vue Error: ${err}\nInfo: ${info}`);
        console.error(err);
    };

    app.use(pinia);
    app.mount("#app");
} catch (e) {
    alert(`Mount Error: ${e}`);
}
