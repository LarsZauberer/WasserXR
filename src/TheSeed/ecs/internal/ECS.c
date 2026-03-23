#include "Scene_internal.h"
#include "TheSeed/ecs/Scene.h"
#include "dlfcn.h"
#include <TheSeed/core/logging.h>
#include <TheSeed/core/utils.h>
#include <glib.h>

static int ts_compare_systems_priority(gconstpointer left,
                                       gconstpointer right) {
  const TS_System_Handler *system_a = *(const TS_System_Handler **)left;
  const TS_System_Handler *system_b = *(const TS_System_Handler **)right;

  return system_a->priority - system_b->priority;
}

void ts_sort_systems(TS_Scene *scene) {
  g_array_sort(scene->systems, ts_compare_systems_priority);
}

void *ts_get_abi_symbol_from_plugin(const TS_Scene *scene,
                                    const TS_Plugin_Handler *handler,
                                    const char *prefix, const char *symbol) {
  ts_assert(prefix, "Prefix is NULL during ts_get_abi_symbol_from_plugin");
  ts_assert(symbol, "Symbol is NULL during ts_get_abi_symbol_from_plugin");

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

void *ts_get_abi_symbol(TS_Plugin_Handler **handler, const TS_Scene *scene,
                        const char *prefix, const char *symbol) {
  ts_assert(prefix, "Prefix is NULL during ts_get_abi_symbol");
  ts_assert(symbol, "Symbol is NULL during ts_get_abi_symbol");
  if (!scene) {
    *handler = NULL;
    return NULL;
  }
  // Check static linking
  void *func = func =
      ts_get_abi_symbol_from_plugin(scene, NULL, prefix, symbol);
  if (func) {
    *handler = NULL;
    return func;
  }
  // Check dynamic linking
  for (size_t i = 0; i < scene->plugins->len; i++) {
    TS_Plugin_Handler *plugin =
        g_array_index(scene->plugins, TS_Plugin_Handler *, i);
    void *func = ts_get_abi_symbol_from_plugin(scene, plugin, prefix, symbol);
    if (func) {
      *handler = plugin;
      return func;
    }
  }
  *handler = NULL;
  return NULL;
}

void ts_reload_plugins(TS_Scene *scene) {
  size_t num_plugins = scene->plugins->len;
  char **plugins_to_load = (char **)malloc(sizeof(char *) * num_plugins);
  for (size_t i = 0; i < num_plugins; i++) {
    TS_Plugin_Handler *plugin_handler =
        g_array_index(scene->plugins, TS_Plugin_Handler *, 0);
    plugins_to_load[i] = ts_copy_char_ptr(plugin_handler->path);
    ts_unload_plugin(scene, plugins_to_load[i]);
  }
  // All Plugins should now be unloaded
  if (scene->plugins->len != 0) {
    ts_warn(
        "Failed to unload all the plugins during the reload of the plugins");
  }
  for (size_t i = 0; i < num_plugins; i++) {
    int status = ts_load_plugin(scene, plugins_to_load[i]);
    if (!status) {
      ts_warn("Failed to reload the plugin `%s` after unloading it",
              plugins_to_load[i]);
    }
    free(plugins_to_load[i]);
  }
  free(plugins_to_load);
}
