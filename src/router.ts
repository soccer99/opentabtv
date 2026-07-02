import { createRouter, createWebHistory } from "vue-router";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      name: "home",
      component: () => import("@/views/HomeView.vue"),
    },
    {
      path: "/live",
      name: "live",
      component: () => import("@/views/LiveTVView.vue"),
    },
    {
      path: "/guide",
      name: "guide",
      component: () => import("@/views/GuideView.vue"),
    },
    {
      path: "/recordings",
      name: "recordings",
      component: () => import("@/views/RecordingsView.vue"),
    },
    {
      path: "/settings",
      name: "settings",
      component: () => import("@/views/SettingsView.vue"),
    },
  ],
});

export default router;
