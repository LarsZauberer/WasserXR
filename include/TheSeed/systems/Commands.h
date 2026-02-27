#include "TheSeed/ecs/Scene.h"

typedef void (*TS_Command_Function)(char **args, TS_Scene *scene);

typedef struct {
  const char *command;
  TS_Command_Function func;
} TS_Command;

void ts_command_reload(char **args, TS_Scene *scene);
void ts_command_exit(char **args, TS_Scene *scene);
void ts_command_removeEntity(char **args, TS_Scene *scene);

TS_Command *ts_create_command_list(size_t *size);
void ts_destroy_command_list(TS_Command *ptr);
