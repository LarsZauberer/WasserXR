#include "TheSeed/systems/ConsoleSystem.h"
#include "TheSeed/components/Commands.h"
#include "TheSeed/components/Console.h"
#include "TheSeed/core/logging.h"
#include "TheSeed/core/utils.h"
#include "TheSeed/ecs/Scene.h"
#include "glib-2.0/glib.h"
#include <glib.h>
#include <pthread.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#define TS_MAX_COMMAND_LENGTH 2048

pthread_t ts_console_thread;
char *ts_console_buffer = NULL;

static char *ts_preprocess_cmd(char *raw) {
  GString *cmd_gstring = g_string_new(raw);
  g_string_truncate(cmd_gstring, cmd_gstring->len - 1);

  char *cmd = g_string_free(cmd_gstring, FALSE);
  return cmd;
}

static void *ts_console_loop(void *arg) {
  while (1) {
    char *buffer_array[TS_MAX_COMMAND_LENGTH];
    char *buffer = (char *)buffer_array;
    size_t bytes_read = read(STDIN_FILENO, buffer, TS_MAX_COMMAND_LENGTH);
    buffer[bytes_read] = '\0';
    ts_info("Console input: %s", buffer);
    if (ts_console_buffer) {
      continue;
    }
    // Preprocessing by removing the \n from the end
    ts_console_buffer = ts_preprocess_cmd(buffer);
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

void ts_system_ts_console_system(TS_Scene *scene, TS_Entity **entities,
                                 const size_t *groups) {
  if (!ts_console_buffer) {
    return;
  }
  if (groups[0] == 0) {
    ts_warn("No entity that is the console");

    free(ts_console_buffer);
    ts_console_buffer = NULL;

    return;
  }
  TS_Console *console_component = (TS_Console *)ts_entity_get_component(
      scene, *(entities[0]), "TS_Console");

  ts_debug("Running command %s", ts_console_buffer);

  size_t cmd_size =
      *(size_t *)ts_get(scene, console_component, "command_list_size");
  TS_Command *cmd_list = ts_get(scene, console_component, "command_list");

  for (size_t i = 0; i < cmd_size; i++) {
    // Check the first command name
    int prefix_status =
        g_str_has_prefix(ts_console_buffer, cmd_list[i].command);
    if (prefix_status) {
      // Get the arguments list
      char *args_begin = ts_console_buffer + strlen(cmd_list[i].command) + 1;
      char **args = g_strsplit(args_begin, " ", -1);
      cmd_list[i].func(args, scene);
      // Clean up
      size_t clear_i = 0;
      while (1) {
        if (!args[clear_i]) {
          break;
        }
        free(args[clear_i++]);
      }
      free(args);
    }
  }

  free(ts_console_buffer);
  ts_console_buffer = NULL;
}

TS_System_Groups ts_select_ts_console_system(TS_Scene *scene,
                                             const TS_Entity entity) {
  if (ts_entity_get_component(scene, entity, "TS_Console")) {
    return 1;
  }
  return 0;
}
