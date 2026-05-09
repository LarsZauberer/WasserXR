#include "WasserXR/core/ConsoleSystem.h"
#include "WasserXR/core/Commands.h"
#include "WasserXR/core/Console.h"
#include "WasserXR/ecs/Scene.h"
#include "WasserXR/ecs/logging.h"
#include "WasserXR/ecs/utils.h"
#include "glib-2.0/glib.h"
#include <glib.h>
#include <pthread.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#define WXR_MAX_COMMAND_LENGTH 2048

static pthread_t wxr_console_thread;
static char *wxr_console_buffer = NULL;

static char *wxr_preprocess_cmd(char *raw) {
  GString *cmd_gstring = g_string_new(raw);
  g_string_truncate(cmd_gstring, cmd_gstring->len - 1);

  char *cmd = g_string_free(cmd_gstring, FALSE);
  return cmd;
}

static void *wxr_console_loop(void *arg) {
  while (1) {
    char *buffer_array[WXR_MAX_COMMAND_LENGTH];
    char *buffer = (char *)buffer_array;
    size_t bytes_read = read(STDIN_FILENO, buffer, WXR_MAX_COMMAND_LENGTH);
    buffer[bytes_read] = '\0';
    wxr_info("Console input: %s", buffer);
    if (wxr_console_buffer) {
      continue;
    }
    // Preprocessing by removing the \n from the end
    wxr_console_buffer = wxr_preprocess_cmd(buffer);
  }
  return NULL;
}

WXR_System_Groups wxr_groups_wxr_console_system = 1;

void wxr_attach_wxr_console_system(WXR_Scene *scene) {
  pthread_create(&wxr_console_thread, NULL, wxr_console_loop, NULL);
}

void wxr_detach_wxr_console_system(WXR_Scene *scene) {
  pthread_cancel(wxr_console_thread);
  pthread_join(wxr_console_thread, NULL);
}

void wxr_system_wxr_console_system(WXR_Scene *scene, WXR_Entity **entities,
                                   const size_t *groups) {
  if (!wxr_console_buffer) {
    return;
  }
  if (groups[0] == 0) {
    wxr_warn("No entity that is the console");

    free(wxr_console_buffer);
    wxr_console_buffer = NULL;

    return;
  }
  WXR_Console *console_component = (WXR_Console *)wxr_entity_get_component(
      scene, *(entities[0]), "WXR_Console");

  wxr_debug("Running command %s", wxr_console_buffer);

  size_t cmd_size =
      *(size_t *)wxr_get(scene, console_component, "command_list_size");
  const WXR_Command *cmd_list =
      wxr_get(scene, console_component, "command_list");

  for (size_t i = 0; i < cmd_size; i++) {
    // Check the first command name
    int prefix_status =
        g_str_has_prefix(wxr_console_buffer, cmd_list[i].command);
    if (prefix_status) {
      // Get the arguments list
      char *args_begin = wxr_console_buffer + strlen(cmd_list[i].command) + 1;
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

  free(wxr_console_buffer);
  wxr_console_buffer = NULL;
}

WXR_System_Groups wxr_select_wxr_console_system(const WXR_Scene *scene,
                                                const WXR_Entity entity) {
  if (wxr_entity_get_component(scene, entity, "WXR_Console")) {
    return 1;
  }
  return 0;
}
