// This is not a system
#include "TheSeed/ecs/Scene.h"

typedef void (*TS_Command_Function)(const char **args, TS_Scene *scene);

typedef struct {
  const char *command;
  TS_Command_Function func;
} TS_Command;

void ts_command_reload(const char **args, TS_Scene *scene);
void ts_command_exit(const char **args, TS_Scene *scene);
void ts_command_addEntity(const char **args, TS_Scene *scene);
void ts_command_removeEntity(const char **args, TS_Scene *scene);
void ts_command_addComponent(const char **args, TS_Scene *scene);
void ts_command_get(const char **args, TS_Scene *scene);
void ts_command_set(const char **args, TS_Scene *scene);
void ts_command_addSystem(const char **args, TS_Scene *scene);
void ts_command_removeSystem(const char **args, TS_Scene *scene);
void ts_command_loadPlugin(const char **args, TS_Scene *scene);
void ts_command_unloadPlugin(const char **args, TS_Scene *scene);
void ts_command_showEntities(const char **args, TS_Scene *scene);
void ts_command_showPlugins(const char **args, TS_Scene *scene);
void ts_command_showComponents(const char **args, TS_Scene *scene);
void ts_command_showSystems(const char **args, TS_Scene *scene);
void ts_command_save(const char **args, TS_Scene *scene);
void ts_command_load(const char **args, TS_Scene *scene);

TS_Command *ts_create_command_list(size_t *size);
void ts_destroy_command_list(TS_Command *ptr);
