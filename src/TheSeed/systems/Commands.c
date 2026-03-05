// This is not a System
#include "TheSeed/systems/Commands.h"
#include "TheSeed/core/logging.h"
#include "TheSeed/ecs/Scene.h"
#include <stdlib.h>
#include <string.h>

TS_Command *ts_create_command_list(size_t *size) {
  *size = 7;
  TS_Command *command_list = (TS_Command *)malloc(sizeof(TS_Command) * *size);

  command_list[0] = (TS_Command){"reload", ts_command_reload};
  command_list[1] = (TS_Command){"exit", ts_command_exit};
  command_list[2] = (TS_Command){"addEntity", ts_command_addEntity};
  command_list[3] = (TS_Command){"removeEntity", ts_command_removeEntity};
  command_list[4] = (TS_Command){"addComponent", ts_command_addComponent};
  command_list[5] = (TS_Command){"get", ts_command_get};
  command_list[6] = (TS_Command){"set", ts_command_set};

  return command_list;
}

void ts_destroy_command_list(TS_Command *ptr) { free(ptr); }

void ts_command_reload(char **args, TS_Scene *scene) {
  ts_set_scene_reload(scene);
}

static int ts_get_window(TS_Scene *scene, TS_Entity entity) {
  if (ts_entity_get_component(scene, entity, "TS_Window")) {
    return 1;
  }
  return 0;
}

void ts_command_exit(char **args, TS_Scene *scene) {
  size_t num_entities = 0;
  TS_Entity *entities = ts_find_entities_with_selector_and_groups(
      &num_entities, scene, ts_get_window, 1);
  for (size_t i = 0; i < num_entities; i++) {
    ts_remove_entity(scene, entities[i]);
  }
  free(entities);
}

void ts_command_addEntity(char **args, TS_Scene *scene) {
  ts_add_entity(scene);
}

void ts_command_removeEntity(char **args, TS_Scene *scene) {
  if (!*args) {
    ts_warn("Remove Entity requires the entity id to remove");
    return;
  }
  size_t entity_id = strtol(args[0], NULL, 10);
  ts_remove_entity(scene, entity_id);
}

void ts_command_addComponent(char **args, TS_Scene *scene) {
  if (!args[0]) {
    ts_warn("Add Component requires the entity id to add to the entity");
    return;
  }
  if (!args[1]) {
    ts_warn("Add Component requires the component");
    return;
  }
  size_t entity_id = strtol(args[0], NULL, 10);
  ts_add_component(scene, entity_id, args[1]);
}

void ts_command_get(char **args, TS_Scene *scene) {
  if (!args[0]) {
    ts_warn("Get requires the entity id to add to the entity");
    return;
  }
  if (!args[1]) {
    ts_warn("Get requires the component");
    return;
  }
  if (!args[2]) {
    ts_warn("Get requires the field name");
    return;
  }
  size_t entity_id = strtol(args[0], NULL, 10);
  void *component = ts_entity_get_component(scene, entity_id, args[1]);
  if (!component) {
    ts_warn("Component `%s` couldn't be found for entity %ld", args[1],
            entity_id);
    return;
  }
  TS_Component_Schema *schema = ts_get_schema_of_component(scene, component);

  TS_Component_Field *field = ts_get_field(schema, args[2]);
  if (!field) {
    ts_warn("Field `%s` was not found in component `%s`", args[2], args[1]);
    return;
  }

  TS_Primitive_Type type = ts_get_field_type(schema, args[2]);

  TS_Component_Getter getter = ts_get_field_getter(schema, args[2]);

  if (!getter) {
    ts_warn("Field `%s` has no getter function", args[2]);
    return;
  }

  void *data = getter(component);
  ts_assert_abort(data, "The getter of the field `%s` returned NULL", args[2]);
  if (type == TS_L) {
    long l_data = *(long *)data;
    ts_info("%s: %ld", args[2], l_data);
  } else if (type == TS_F) {
    float f_data = *(float *)data;
    ts_info("%s: %f", args[2], f_data);
  } else if (type == TS_C) {
    char c_data = *(char *)data;
    ts_info("%s: %c", args[2], c_data);
  } else if (type == TS_BLOB) {
    long l_data = *(long *)data;
    ts_info("%s: 0x%lx", args[2], l_data);
  } else if (type == TS_S) {
    char *s_data = (char *)data;
    ts_info("%s: %s", args[2], s_data);
  } else if (type == TS_BLOB_ARRAY) {
    ts_info("%s: BLOB Array", args[2]);
  } else {
    ts_critical("TS_Primitive_Type is not valid");
  }
}

void ts_command_set(char **args, TS_Scene *scene) {
  if (!args[0]) {
    ts_warn("Set requires the entity id to add to the entity");
    return;
  }
  if (!args[1]) {
    ts_warn("Set requires the component");
    return;
  }
  if (!args[2]) {
    ts_warn("Set requires the field name");
    return;
  }
  if (!args[3]) {
    ts_warn("Set requires a value");
    return;
  }
  size_t entity_id = strtol(args[0], NULL, 10);
  void *component = ts_entity_get_component(scene, entity_id, args[1]);
  if (!component) {
    ts_warn("Component `%s` couldn't be found for entity %ld", args[1],
            entity_id);
    return;
  }
  TS_Component_Schema *schema = ts_get_schema_of_component(scene, component);

  TS_Component_Field *field = ts_get_field(schema, args[2]);
  if (!field) {
    ts_warn("Field `%s` was not found in component `%s`", args[2], args[1]);
    return;
  }

  TS_Primitive_Type type = ts_get_field_type(schema, args[2]);

  TS_Component_Setter setter = ts_get_field_setter(schema, args[2]);

  if (!setter) {
    ts_warn("Field `%s` has no setter function", args[2]);
    return;
  }

  if (type == TS_L) {
    long l_data = strtol(args[3], NULL, 10);
    ts_set(scene, component, args[2], &l_data);
  } else if (type == TS_F) {
    float f_data = strtof(args[3], NULL);
    ts_set(scene, component, args[2], &f_data);
  } else if (type == TS_C) {
    char c_data = *args[3];
    ts_set(scene, component, args[2], &c_data);
  } else if (type == TS_S) {
    ts_set(scene, component, args[2], args[2]);
  } else {
    ts_warn("Cannot handle such a primitive type");
  }
}
