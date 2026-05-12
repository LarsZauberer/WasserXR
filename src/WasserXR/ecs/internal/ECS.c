/*
 * Copyright (c) 2026 Ian Wasser
 * Licensed under the WasserXR License.
 * You may not use this file except in compliance with the License.
 * See LICENSE.md for details.
 */

#include "Scene_internal.h"
#include "WasserXR/ecs/Scene.h"
#include "dlfcn.h"
#include <WasserXR/ecs/logging.h>
#include <WasserXR/ecs/utils.h>
#include <glib.h>

static int wxr_compare_systems_priority(gconstpointer left,
                                        gconstpointer right) {
  const WXR_System_Handler *system_a = *(const WXR_System_Handler **)left;
  const WXR_System_Handler *system_b = *(const WXR_System_Handler **)right;

  return system_a->priority - system_b->priority;
}

void wxr_sort_systems(WXR_Scene *scene) {
  g_array_sort(scene->systems, wxr_compare_systems_priority);
}

void *wxr_get_abi_symbol_from_plugin(const WXR_Scene *scene,
                                     const WXR_Plugin_Handler *handler,
                                     const char *prefix, const char *symbol) {
  wxr_assert(prefix, "Prefix is NULL during wxr_get_abi_symbol_from_plugin");
  wxr_assert(symbol, "Symbol is NULL during wxr_get_abi_symbol_from_plugin");

  if (!scene) {
    return NULL;
  }

  GString *working_symbol = g_string_new(symbol);
  g_string_prepend(working_symbol, prefix);
  void *func;
  if (handler) {
    func = dlsym(handler->fd, working_symbol->str);
  } else {
    func = dlsym(RTLD_DEFAULT, working_symbol->str);
  }
  g_string_free(working_symbol, TRUE);
  if (func) {
    return func;
  }
  return NULL;
}

void *wxr_get_abi_symbol(WXR_Plugin_Handler **handler, const WXR_Scene *scene,
                         const char *prefix, const char *symbol) {
  wxr_assert(prefix, "Prefix is NULL during wxr_get_abi_symbol");
  wxr_assert(symbol, "Symbol is NULL during wxr_get_abi_symbol");
  if (!scene) {
    *handler = NULL;
    return NULL;
  }
  // Check static linking
  void *func = wxr_get_abi_symbol_from_plugin(scene, NULL, prefix, symbol);
  if (func) {
    *handler = NULL;
    return func;
  }
  // Check dynamic linking
  for (size_t i = 0; i < scene->plugins->len; i++) {
    WXR_Plugin_Handler *plugin =
        g_array_index(scene->plugins, WXR_Plugin_Handler *, i);
    void *func = wxr_get_abi_symbol_from_plugin(scene, plugin, prefix, symbol);
    if (func) {
      *handler = plugin;
      return func;
    }
  }
  *handler = NULL;
  return NULL;
}

void wxr_reload_plugins(WXR_Scene *scene) {
  size_t num_plugins = scene->plugins->len;
  char **plugins_to_load = (char **)malloc(sizeof(char *) * num_plugins);
  for (size_t i = 0; i < num_plugins; i++) {
    WXR_Plugin_Handler *plugin_handler =
        g_array_index(scene->plugins, WXR_Plugin_Handler *, 0);
    plugins_to_load[i] = wxr_copy_char_ptr(plugin_handler->path);
    wxr_unload_plugin(scene, plugins_to_load[i]);
  }
  // All Plugins should now be unloaded
  if (scene->plugins->len != 0) {
    wxr_warn(
        "Failed to unload all the plugins during the reload of the plugins");
  }
  for (size_t i = 0; i < num_plugins; i++) {
    int status = wxr_load_plugin(scene, plugins_to_load[i]);
    if (status) {
      wxr_warn("Failed to reload the plugin `%s` after unloading it",
               plugins_to_load[i]);
    }
    free(plugins_to_load[i]);
  }
  free(plugins_to_load);
}
