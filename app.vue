<!-- SPDX-FileCopyrightText: Copyright (c) 2022-2026 trobonox <hello@trobo.dev> -->
<!-- -->
<!-- SPDX-License-Identifier: CC0-1.0 -->

<template>
  <div>
    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>
  </div>
</template>

<script setup>
import {
  attachConsole,
  trace,
  debug,
  info,
  warn,
  error,
} from "@tauri-apps/plugin-log";
import { listen } from "@tauri-apps/api/event";
import { useBoardsStore } from "@/stores/boards";

onMounted(async () => {
  await attachConsole();

  console.trace = trace;
  console.log = debug;
  console.info = info;
  console.warn = warn;
  console.error = error;

  const boardsStore = useBoardsStore();

  await listen("kanri-file-changed", async () => {
    console.info("External file change detected, reloading boards...");
    await boardsStore.forceReloadBoards();
  });
});
</script>
