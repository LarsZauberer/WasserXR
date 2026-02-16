#include "TheSeed/components/Model.h"
#include "TheSeed/core/Mesh.h"
#include "TheSeed/core/Shader.h"
#include "TheSeed/ecs/Scene.h"
#include <glad/gl.h>
#include <stdlib.h>
#include <string.h>

void *ts_create_TS_Model() {
  TS_Model *model = (TS_Model *)malloc(sizeof(TS_Model));

  model->model_name = NULL;
  model->shader_name = NULL;
  model->meshes = NULL;

  return model;
}

void ts_destroy_TS_Mesh(void *p) {
  TS_Model *model = (TS_Model *)p;
  free(model->model_name);
  free(model->shader_name);
  free(model->meshes);
  ts_destroy_shader(model->shader);
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
    // TODO: Fix the issue with the double pointering
    TS_Mesh_Data **mesh_data = NULL;
    unsigned int n = ts_read_mesh_data(mesh_data, model->model_name);

    // Load the mesh with opengl
    for (unsigned int i = 0; i < n; i++) {
      ts_create_mesh_from_data(mesh_data[i]);
    }
  }
}
