#include "Scene_internal.h"
#include "TheSeed/ecs/Scene.h"
#include <TheSeed/core/logging.h>
#include <TheSeed/core/utils.h>

TS_Component_Serialization *
ts_create_component_serialization(const char *component_name) {
  TS_Component_Serialization *serialization =
      (TS_Component_Serialization *)malloc(sizeof(TS_Component_Serialization));
  serialization->component_name = ts_copy_char_ptr(component_name);
  serialization->fields =
      g_array_new(FALSE, FALSE, sizeof(TS_Component_Serialization_Item *));
  return serialization;
}

TS_Component_Serialization_Item *
ts_create_component_serialization_item(const char *field_name, const void *data,
                                       const size_t size,
                                       const TS_Primitive_Type type) {
  TS_Component_Serialization_Item *field =
      (TS_Component_Serialization_Item *)malloc(
          sizeof(TS_Component_Serialization_Item));
  field->field_name = ts_copy_char_ptr(field_name);
  field->size = size;

  if (type == TS_S || type == TS_BLOB_ARRAY) {
    // Handling of pointer field types that might have multiple elements.
    // Note that all the arrays have to be NULL terminated
    void *data_loc = ts_memcpy_till_null(data, size);
    field->data = data_loc;
  } else {
    // Handling of standard single value fields
    void *data_loc = malloc(size);
    memcpy(data_loc, data, size);
    field->data = data_loc;
  }

  return field;
}

void ts_destroy_component_serialization_item(
    TS_Component_Serialization_Item *item) {
  free(item->field_name);
  free(item->data);
  free(item);
}

void ts_destroy_component_serialization(
    TS_Component_Serialization *serialization) {
  for (size_t i = 0; i < serialization->fields->len; i++) {
    TS_Component_Serialization_Item *field = g_array_index(
        serialization->fields, TS_Component_Serialization_Item *, i);
    ts_destroy_component_serialization_item(field);
  }
  g_array_free(serialization->fields, TRUE);
  free(serialization->component_name);
  free(serialization);
}

TS_Component_Serialization *
ts_serialize_component_internal(const TS_Component_Handler *handler) {
  TS_Component_Serialization *serialization =
      ts_create_component_serialization(handler->id);

  ts_assert(handler->schema,
            "Component `%s` has no schema during serialization", handler->id);

  // Gather all the data from all the fields that are serializable and copy them
  // into a serialization_item
  for (size_t i = 0; i < handler->schema->fields->len; i++) {
    TS_Component_Field *field =
        g_array_index(handler->schema->fields, TS_Component_Field *, i);

    if (!field->getter) {
      continue;
    }
    void *data = field->getter(handler->component);
    TS_Component_Serialization_Item *serialization_item =
        ts_create_component_serialization_item(field->field_name, data,
                                               field->size, field->type);
    g_array_append_val(serialization->fields, serialization_item);
  }

  return serialization;
}

int ts_deserialize_component_internal(
    TS_Scene *scene, const TS_Entity entity,
    const TS_Component_Serialization *serialization) {
  // Create component
  int status = ts_add_component(scene, entity, serialization->component_name);
  ts_assert_abort_value(!status, 1,
                        "Failed to add component `%s` to entity %ld during "
                        "internal component deserialization",
                        serialization->component_name, entity);
  void *handler =
      ts_entity_get_component(scene, entity, serialization->component_name);
  ts_assert_abort_value(handler, 1,
                        "Failed to get the added component `%s` from entity "
                        "%ld during internal component deserialization",
                        serialization->component_name, entity);
  // Performs all the setter with the data
  // Note that the setter in the user code is responsible for a potential copy
  // of the value
  status = 0;
  for (size_t i = 0; i < serialization->fields->len; i++) {
    TS_Component_Serialization_Item *item = g_array_index(
        serialization->fields, TS_Component_Serialization_Item *, i);
    status |= ts_set(scene, handler, item->field_name, item->data);
  }
  return status;
}
