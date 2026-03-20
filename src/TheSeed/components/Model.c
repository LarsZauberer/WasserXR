#include "TheSeed/components/Model.h"
#include "TheSeed/core/Mesh.h"
#include "TheSeed/core/Mesh_Data.h"
#include "TheSeed/core/Shader.h"
#include "TheSeed/core/logging.h"
#include "TheSeed/core/utils.h"
#include "TheSeed/ecs/Macros.h"
#include "TheSeed/ecs/Scene.h"
#include <glad/gl.h>
#include <stdlib.h>
#include <string.h>

struct TS_Model {
  char *model_name;
  char *shader_name;
  TS_Shader *shader;
  unsigned int numMeshes;
  TS_Mesh **meshes;
};

void *ts_create_TS_Model() {
  TS_Model *model = (TS_Model *)malloc(sizeof(TS_Model));
  ts_assert(model, "Malloc failed during ts_create_TS_Model");

  model->model_name = NULL;
  model->shader_name = NULL;
  model->meshes = NULL;
  model->shader = NULL;
  model->numMeshes = 0;

  return model;
}

void ts_destroy_TS_Model(void *ptr) {
  TS_Model *model = (TS_Model *)ptr;
  free(model->model_name);
  free(model->shader_name);

  // Free all the meshes
  for (unsigned int i = 0; i < model->numMeshes; i++) {
    ts_destroy_mesh(model->meshes[i]);
  }
  free(model->meshes);

  // Free the shader
  ts_destroy_shader(model->shader);

  // Free the model
  free(model);
}

TS_STRING_SERIALIZE(TS_Model, model_name, component->model_name);
TS_SET_DESERIALIZE(TS_Model, model_name, component->model_name,
                   ts_set_TS_Model_model_name);
TS_STRING_SERIALIZE(TS_Model, shader_name, component->shader_name);
TS_SET_DESERIALIZE(TS_Model, shader_name, component->shader_name,
                   ts_set_TS_Model_shader_name);

void ts_schema_TS_Model(TS_Component_Schema *schema) {
  TS_Component_Field *model_name_field = ts_create_component_field(
      "model_name", sizeof(char), TS_S, ts_get_TS_Model_model_name,
      ts_set_TS_Model_model_name, ts_serialize_TS_Model_model_name,
      ts_deserialize_TS_Model_model_name);
  TS_Component_Field *shader_name_field = ts_create_component_field(
      "shader_name", sizeof(char), TS_S, ts_get_TS_Model_shader_name,
      ts_set_TS_Model_shader_name, ts_serialize_TS_Model_shader_name,
      ts_deserialize_TS_Model_shader_name);

  TS_Component_Field *meshes_field =
      ts_create_component_field("meshes", sizeof(TS_Mesh *), TS_BLOB_ARRAY,
                                ts_get_TS_Model_meshes, NULL, NULL, NULL);
  TS_Component_Field *numMeshes_field =
      ts_create_component_field("num_meshes", sizeof(unsigned int), TS_L,
                                ts_get_TS_Model_numMeshes, NULL, NULL, NULL);

  TS_Component_Field *shader_field =
      ts_create_component_field("shader", sizeof(TS_Shader *), TS_BLOB,
                                ts_get_TS_Model_shader, NULL, NULL, NULL);

  ts_add_field_to_component_schema(schema, model_name_field);
  ts_add_field_to_component_schema(schema, shader_name_field);
  ts_add_field_to_component_schema(schema, meshes_field);
  ts_add_field_to_component_schema(schema, numMeshes_field);
  ts_add_field_to_component_schema(schema, shader_field);
}

void ts_set_TS_Model_model_name(void *component, void *data) {
  TS_Model *model = (TS_Model *)component;
  char *path = (char *)data;
  if (path) {
    // Replace the field
    free(model->model_name);
    model->model_name = ts_copy_char_ptr(path);

    // Read all the mesh data (array)
    TS_Mesh_Data *mesh_data =
        ts_read_mesh_data(&model->numMeshes, model->model_name);
    if (!mesh_data) {
      ts_warn("Failed to read the mesh data of `%s`", model->model_name);
    }

    // Array of pointers
    TS_Mesh **meshes = (TS_Mesh **)malloc(sizeof(TS_Mesh *) * model->numMeshes);
    ts_assert(meshes, "Malloc failed during creation of the meshes array in "
                      "ts_activate_TS_Model");

    // Load the mesh with opengl
    for (unsigned int i = 0; i < model->numMeshes; i++) {
      meshes[i] = ts_create_mesh_from_data(&mesh_data[i]);
      // Free up the mesh data
      ts_destroy_mesh_data(&mesh_data[i]);
    }
    free(mesh_data);

    model->meshes = meshes;
  } else {
    // TODO: Clean the old data and set null
  }
}

void ts_set_TS_Model_shader_name(void *component, void *data) {
  TS_Model *model = (TS_Model *)component;
  char *path = (char *)data;
  if (path) {
    // Replace the field
    free(model->shader_name);
    model->shader_name = ts_copy_char_ptr(path);

    model->shader = ts_create_shader(model->shader_name);
    int status = ts_load_shader(model->shader);
    if (status) {
      ts_warn("Failed to load the shader: %s", model->shader_name);
    } else {
      status = ts_compile_shader(model->shader);
      if (status) {
        ts_warn("Failed to compile the shader: %s", model->shader_name);
      }
    }
  } else {
    // TODO: Clean the old data and set null
  }
}

void *ts_get_TS_Model_shader_name(void *component) {
  TS_Model *model = (TS_Model *)component;
  return model->shader_name;
}

void *ts_get_TS_Model_model_name(void *component) {
  TS_Model *model = (TS_Model *)component;
  return model->model_name;
}

void *ts_get_TS_Model_shader(void *component) {
  TS_Model *model = (TS_Model *)component;
  return model->shader;
}

void *ts_get_TS_Model_meshes(void *component) {
  TS_Model *model = (TS_Model *)component;
  return model->meshes;
}

void *ts_get_TS_Model_numMeshes(void *component) {
  TS_Model *model = (TS_Model *)component;
  return &model->numMeshes;
}
