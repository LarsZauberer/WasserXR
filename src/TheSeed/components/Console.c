#include "TheSeed/components/Console.h"
#include "TheSeed/core/Commands.h"
#include "TheSeed/ecs/Scene.h"

#include <stdlib.h>
struct TS_Console {
  size_t command_list_size;
  TS_Command *command_list;
};

void *ts_create_TS_Console() {
  TS_Console *console = (TS_Console *)malloc(sizeof(TS_Console));
  console->command_list = ts_create_command_list(&console->command_list_size);
  return console;
}

void ts_destroy_TS_Console(void *ptr) {
  TS_Console *console = (TS_Console *)ptr;
  ts_destroy_command_list(console->command_list);
  free(console);
}

void ts_schema_TS_Console(TS_Component_Schema *schema) {
  TS_Component_Field *command_list_size_field = ts_create_component_field(
      "command_list_size", sizeof(size_t), TS_L, TS_Permission_No_Serialize,
      ts_get_TS_Console_command_list_size, NULL);
  TS_Component_Field *command_list_field = ts_create_component_field(
      "command_list", sizeof(TS_Command), TS_BLOB_ARRAY,
      TS_Permission_No_Serialize, ts_get_TS_Console_command_list, NULL);

  ts_add_field_to_component_schema(schema, command_list_size_field);
  ts_add_field_to_component_schema(schema, command_list_field);
}

void *ts_get_TS_Console_command_list_size(void *component) {
  TS_Console *console = (TS_Console *)component;
  return &console->command_list_size;
}

void *ts_get_TS_Console_command_list(void *component) {
  TS_Console *console = (TS_Console *)component;
  return console->command_list;
}
