#include "TheSeed/ecs/Scene.h"

typedef void (*TS_Command_Function)(char **args, TS_Scene *scene);

typedef struct {
  const char *command;
  TS_Command_Function func;
} TS_Command;

void ts_command_reload(char **args, TS_Scene *scene);
void ts_command_exit(char **args, TS_Scene *scene);
void ts_command_addEntity(char **args, TS_Scene *scene);
void ts_command_removeEntity(char **args, TS_Scene *scene);
void ts_command_addComponent(char **args, TS_Scene *scene);
void ts_command_get(char **args, TS_Scene *scene);
void ts_command_set(char **args, TS_Scene *scene);
void ts_command_addSystem(char **args, TS_Scene *scene);
void ts_command_removeSystem(char **args, TS_Scene *scene);
void ts_command_loadPlugin(char **args, TS_Scene *scene);
void ts_command_unloadPlugin(char **args, TS_Scene *scene);
void ts_command_showEntities(char **args, TS_Scene *scene);
void ts_command_showPlugins(char **args, TS_Scene *scene);
void ts_command_showComponents(char **args, TS_Scene *scene);
void ts_command_showSystems(char **args, TS_Scene *scene);
void ts_command_export(char **args, TS_Scene *scene);
void ts_command_save(char **args, TS_Scene *scene);
void ts_command_load(char **args, TS_Scene *scene);

TS_Command *ts_create_command_list(size_t *size);
void ts_destroy_command_list(TS_Command *ptr);
