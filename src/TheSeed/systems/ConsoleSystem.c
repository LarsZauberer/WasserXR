#include "TheSeed/systems/ConsoleSystem.h"
#include "TheSeed/core/logging.h"
#include "TheSeed/core/utils.h"
#include "TheSeed/ecs/Scene.h"
#include "glib-2.0/glib.h"
#include <glib.h>
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#define TS_MAX_COMMAND_LENGTH 265

pthread_t ts_console_thread;
char *console_buffer = NULL;

static char *ts_preprocess_cmd(char *raw) {
  GString *cmd_gstring = g_string_new(raw);
  g_string_truncate(cmd_gstring, cmd_gstring->len - 1);

  char *cmd = g_string_free(cmd_gstring, FALSE);
  return cmd;
}

static void *ts_console_loop(void *arg) {
  while (1) {
    char *buffer = (char *)malloc(sizeof(char) * TS_MAX_COMMAND_LENGTH);
    size_t bytes_read = read(STDIN_FILENO, buffer, TS_MAX_COMMAND_LENGTH);
    buffer[bytes_read] = '\0';
    ts_info("Console input: %s", buffer);
    if (console_buffer) {
      free(buffer);
      continue;
    }
    // Preprocessing by removing the \n from the end
    console_buffer = ts_preprocess_cmd(buffer);
    free(buffer);
  }
  return NULL;
}

void ts_attach_ts_console_system(TS_Scene *scene) {
  pthread_create(&ts_console_thread, NULL, ts_console_loop, NULL);
}

void ts_detach_ts_console_system(TS_Scene *scene) {
  pthread_cancel(ts_console_thread);
  pthread_join(ts_console_thread, NULL);
}

int ts_select_ts_console_system(TS_Scene *scene, TS_Entity entity) { return 0; }

static int ts_get_window(TS_Scene *scene, TS_Entity entity) {
  if (ts_entity_get_component(scene, entity, "TS_Window")) {
    return 1;
  }
  return 0;
}

void ts_system_ts_console_system(TS_Scene *scene, TS_Entity **entities,
                                 const size_t *groups) {
  if (!console_buffer) {
    return;
  }

  ts_debug("Running command %s", console_buffer);

  int removeEntity = g_str_has_prefix(console_buffer, "removeEntity");

  if (strcmp(console_buffer, "reload") == 0) {
    ts_set_scene_reload(scene);
  } else if (strcmp(console_buffer, "exit") == 0) {
    size_t size = 0;
    TS_Entity *windows = ts_find_entities_with_selector_and_groups(
        &size, scene, ts_get_window, 1);
    for (size_t i = 0; i < size; i++) {
      ts_remove_entity(scene, windows[i]);
    }

    free(windows);
  } else if (strcmp(console_buffer, "addEntity") == 0) {
    TS_Entity entity_id = ts_add_entity(scene);
    ts_info("Entity with id %ld created", entity_id);
  } else if (removeEntity) {
    char *args = console_buffer + strlen("removeEntity ");
    size_t entity_id = strtol(args, NULL, 10);
    ts_remove_entity(scene, entity_id);
  }

  free(console_buffer);
  console_buffer = NULL;
}
