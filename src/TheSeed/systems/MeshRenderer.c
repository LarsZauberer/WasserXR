#include "glad/gl.h"

#include "GLFW/glfw3.h"
#include "TheSeed/components/Camera.h"
#include "TheSeed/components/Model.h"
#include "TheSeed/components/Transform.h"
#include "TheSeed/components/Window.h"
#include "TheSeed/core/Shader.h"
#include "TheSeed/ecs/Scene.h"
#include "cglm/affine-pre.h"
#include "cglm/affine.h"
#include "cglm/cam.h"
#include "cglm/mat4.h"
#include "cglm/types.h"
#include "cglm/util.h"
#include "cglm/vec4.h"
#include <stdio.h>

#define FOV 90.0f
#define NEAR 0.1f
#define FAR 100.0f

int ts_select_ts_mesh_renderer(TS_Scene_t *scene, const TS_Entity entity) {
  size_t normal_object =
      ts_entity_get_component(scene, entity, "TS_Transform") &&
      ts_entity_get_component(scene, entity, "TS_Model");
  size_t camera_object =
      ts_entity_get_component(scene, entity, "TS_Transform") &&
      ts_entity_get_component(scene, entity, "TS_Camera");
  size_t window = (size_t)ts_entity_get_component(scene, entity, "TS_Window");
  if (normal_object || camera_object || window) {
    return 1;
  } else {
    return 0;
  }
}

void ts_system_ts_mesh_renderer(TS_Scene_t *scene, TS_Entity *entities,
                                size_t n) {
  TS_Entity camera_entity;
  TS_Camera *cam;
  TS_Transform_t *cam_transform;

  TS_Entity window_entity;
  TS_Window *window;

  // Find the camera
  for (size_t i = 0; i < n; i++) {
    TS_Entity entity = entities[i];
    cam = ts_entity_get_component(scene, entity, "TS_Camera");
    cam_transform = ts_entity_get_component(scene, entity, "TS_Transform");
    if (cam) {
      camera_entity = entity;
      break;
    }
  }
  if (!cam) {
    // No camera found
    printf("No camera found!\n");
    return;
  }

  // Find the window
  for (size_t i = 0; i < n; i++) {
    TS_Entity entity = entities[i];
    window = ts_entity_get_component(scene, entity, "TS_Window");
    if (window) {
      window_entity = entity;
      break;
    }
  }
  if (!window) {
    // No window found
    printf("No window found!\n");
    return;
  }

  for (size_t i = 0; i < n; i++) {
    // Normal mesh entity
    TS_Entity entity = entities[i];
    if (entity == camera_entity || entity == window_entity) {
      continue;
    }
    TS_Model *model = ts_entity_get_component(scene, entity, "TS_Model");
    TS_Transform_t *transform =
        ts_entity_get_component(scene, entity, "TS_Transform");

    // Check if the model is loaded yet
    if (model->meshes == NULL || model->shader == NULL) {
      continue;
    }

    ts_use_shader(model->shader);
    mat4 model_transform;
    mat4 view_transform;
    mat4 projection_transform;
    glm_mat4_identity(model_transform);
    glm_mat4_identity(view_transform);
    glm_mat4_identity(projection_transform);

    // Create the transformation matrix

    // World Space placement
    glm_translate(model_transform, transform->position);
    glm_rotate_x(model_transform, glm_rad(transform->rotation[0]),
                 model_transform);
    glm_rotate_y(model_transform, glm_rad(transform->rotation[1]),
                 model_transform);
    glm_rotate_z(model_transform, glm_rad(transform->rotation[2]),
                 model_transform);
    glm_scale(model_transform, transform->scale);

    // Camera placement
    vec4 camera_pos_4;
    glm_vec4(cam_transform->position, 1.0f, camera_pos_4);
    glm_vec4_negate(camera_pos_4);
    glm_translate(view_transform, camera_pos_4);

    // Perspective
    int width, height;
    glfwGetWindowSize(window->window, &width, &height);
    glm_perspective(glm_rad(FOV), (float)width / (float)height, NEAR, FAR,
                    projection_transform);

    // Put everything to the respective uniforms in the shader
    ts_set_shader_uniform_mat4(model->shader, "model", model_transform);
    ts_set_shader_uniform_mat4(model->shader, "view", view_transform);
    ts_set_shader_uniform_mat4(model->shader, "projection",
                               projection_transform);

    // Draw the meshes
    for (unsigned int i = 0; i < model->numMeshes; i++) {
      glBindVertexArray(model->meshes[i]->vao);

      glDrawElements(GL_TRIANGLES, model->meshes[i]->numIndices,
                     GL_UNSIGNED_INT, 0);
    }
  }
  return;
}
