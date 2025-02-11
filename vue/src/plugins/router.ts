import { createRouter, createWebHistory } from "vue-router";

import DeviceInfo from "../components/page/DeviceInfo.vue";
import Login from "../components/page/Login.vue";

const routes = [
	{ path: "/", component: DeviceInfo },
	{ path: "/login", component: Login },
];

const router = createRouter({
	history: createWebHistory(),
	routes,
});

export default router;
