#include "TheSeed/components/Console.h"
#include "TheSeed/components/Commands.h"
#include "TheSeed/ecs/Macros.h"
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

TS_BASIC_GETTER(TS_Console, command_list_size, &component->command_list_size,
                sizeof(size_t));
TS_BASIC_GETTER(TS_Console, command_list, component->command_list,
                sizeof(TS_Command *));

void ts_schema_TS_Console(TS_Component_Schema *schema) {
  TS_SCHEMA_FIELD_GET(TS_Console, TS_L, command_list_size);
  TS_SCHEMA_FIELD_GET(TS_Console, TS_BLOB, command_list);
}
