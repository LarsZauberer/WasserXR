#include "TheSeed/components/Model.h"
#include "TheSeed/core/Mesh.h"
#include "TheSeed/core/Shader.h"
#include "TheSeed/core/logging.h"
#include "TheSeed/core/utils.h"
#include <glad/gl.h>
#include <stdlib.h>
#include <string.h>

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

void ts_destroy_TS_Model(void *p) {
  TS_Model *model = (TS_Model *)p;
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
  return;
}

void ts_serialize_TS_Model(void *ptr, TS_Serialization *serialization) {
  TS_Model *model = (TS_Model *)ptr;

  if (!ptr || !serialization) {
    return;
  }

  if (model->model_name) {
    char *model_name_copy = ts_copy_char_ptr(model->model_name);
    ts_set_serialization(serialization, "model_name",
                         sizeof(char) * (strlen(model->model_name) + 1),
                         model_name_copy);
  }

  if (model->shader_name) {
    char *shader_name_copy = ts_copy_char_ptr(model->shader_name);
    ts_set_serialization(serialization, "shader_name",
                         sizeof(char) * (strlen(model->shader_name) + 1),
                         shader_name_copy);
  }
  return;
}

void ts_deserialize_TS_Model(void *ptr, TS_Serialization *serialization) {
  TS_Model *model = (TS_Model *)ptr;

  if (!ptr || !serialization) {
    return;
  }

  if (ts_get_serialization(serialization, "model_name")) {
    char *serialization_model_name =
        (char *)ts_get_serialization(serialization, "model_name");

    model->model_name = ts_copy_char_ptr(serialization_model_name);
  }

  if (ts_get_serialization(serialization, "shader_name")) {
    char *serialization_shader_name =
        (char *)ts_get_serialization(serialization, "shader_name");

    model->shader_name = ts_copy_char_ptr(serialization_shader_name);
  }
  return;
}

void ts_activate_TS_Model(void *ptr) {
  ts_assert(ptr, "Pointer passed to ts_activate_TS_Model");
  TS_Model *model = (TS_Model *)ptr;

  if (model->shader_name) {
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
  }

  if (model->model_name) {
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
  }
  return;
}
