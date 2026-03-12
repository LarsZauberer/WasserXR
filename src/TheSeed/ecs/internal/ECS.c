#include "Scene_internal.h"
#include "TheSeed/ecs/Scene.h"
#include "dlfcn.h"
#include <TheSeed/core/logging.h>
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

  if (!scene || !handler) {
    return NULL;
  }

  GString *working_symbol = g_string_new(symbol);
  g_string_prepend(working_symbol, prefix);
  void *func = dlsym(handler->fd, working_symbol->str);
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
