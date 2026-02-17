#include "TheSeed/components/Model.h"
#include "TheSeed/core/Mesh.h"
#include "TheSeed/core/Shader.h"
#include "TheSeed/ecs/Scene.h"
#include <glad/gl.h>
#include <stdlib.h>
#include <string.h>

#define CHECKED_FREE(ptr)                                                      \
  if (ptr) {                                                                   \
    free(ptr);                                                                 \
  }

void *ts_create_TS_Model() {
  TS_Model *model = (TS_Model *)malloc(sizeof(TS_Model));

  model->model_name = NULL;
  model->shader_name = NULL;
  model->meshes = NULL;
  model->shader = NULL;
  model->numMeshes = 0;

  return model;
}

void ts_destroy_TS_Model(void *p) {
  TS_Model *model = (TS_Model *)p;
  CHECKED_FREE(model->model_name);
  CHECKED_FREE(model->shader_name);

  // Free all the meshes
  for (unsigned int i = 0; i < model->numMeshes; i++) {
    ts_destroy_mesh(model->meshes[i]);
  }
  CHECKED_FREE(model->meshes);

  // Free the shader
  ts_destroy_shader(model->shader);

  // Free the model
  free(model);
  return;
}

void ts_serialize_TS_Model(void *ptr, TS_Serialization *serialization) {
  TS_Model *model = (TS_Model *)ptr;

  if (!ptr || !serialization) {
    return;
  }

  if (model->model_name) {
    ts_set_serialization(serialization, "model_name",
                         sizeof(char) * strlen(model->model_name),
                         model->model_name);
  }

  if (model->shader_name) {
    ts_set_serialization(serialization, "shader_name",
                         sizeof(char) * strlen(model->shader_name),
                         model->shader_name);
  }
  return;
}

void ts_deserialize_TS_Model(void *ptr, TS_Serialization *serialization) {
  TS_Model *model = (TS_Model *)ptr;

  if (!ptr || !serialization) {
    return;
  }

  if (ts_get_serialization(serialization, "model_name")) {
    model->model_name =
        (char *)ts_get_serialization(serialization, "model_name");
  }

  if (ts_get_serialization(serialization, "shader_name")) {
    model->shader_name =
        (char *)ts_get_serialization(serialization, "shader_name");
  }
  ts_activate_TS_Model(model);
  return;
}

void ts_activate_TS_Model(void *ptr) {
  TS_Model *model = (TS_Model *)ptr;

  if (!model) {
    return;
  }

  if (model->shader_name) {
    model->shader = ts_create_shader(model->shader_name);
  }

  if (model->model_name) {
    // Read all the mesh data
    TS_Mesh_Data *mesh_data = NULL;
    model->numMeshes = ts_read_mesh_data(mesh_data, model->model_name);

    TS_Mesh **meshes = (TS_Mesh **)malloc(sizeof(TS_Mesh) * model->numMeshes);

    // Load the mesh with opengl
    for (unsigned int i = 0; i < model->numMeshes; i++) {
      meshes[i] = ts_create_mesh_from_data(&mesh_data[i]);
    }

    model->meshes = meshes;
  }
  return;
}
