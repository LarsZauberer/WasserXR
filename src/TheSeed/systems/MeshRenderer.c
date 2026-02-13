#include "glad/gl.h"

#include "GLFW/glfw3.h"
#include "TheSeed/components/Camera.h"
#include "TheSeed/components/Mesh.h"
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

int ts_select_ts_mesh_renderer(TS_Scene_t *scene, const size_t entity) {
  size_t normal_object =
      ts_entity_get_component(scene, entity, "TS_Transform") &&
      ts_entity_get_component(scene, entity, "TS_Mesh");
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

void ts_system_ts_mesh_renderer(TS_Scene_t *scene, size_t *entities, size_t n) {
  size_t camera_entity;
  TS_Camera *cam;
  TS_Transform_t *cam_transform;

  size_t window_entity;
  TS_Window *window;

  // Find the camera
  for (size_t i = 0; i < n; i++) {
    size_t entity = entities[i];
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
    size_t entity = entities[i];
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
    // Entity has to have both mesh and transform
    size_t entity = entities[i];
    if (entity == camera_entity || entity == window_entity) {
      continue;
    }
    TS_Mesh *mesh = ts_entity_get_component(scene, entity, "TS_Mesh");
    TS_Transform_t *transform =
        ts_entity_get_component(scene, entity, "TS_Transform");

    glBindVertexArray(mesh->vao);
    ts_use_shader(mesh->shader);

    // Create the transformation matrix
    mat4 model;
    mat4 view;
    mat4 projection;
    glm_mat4_identity(model);
    glm_mat4_identity(view);
    glm_mat4_identity(projection);

    // World Space placement
    glm_translate(model, transform->position);
    glm_rotate_x(model, glm_rad(transform->rotation[0]), model);
    glm_rotate_y(model, glm_rad(transform->rotation[1]), model);
    glm_rotate_z(model, glm_rad(transform->rotation[2]), model);
    glm_scale(model, transform->scale);

    // Camera placement
    vec4 camera_pos_4;
    glm_vec4(cam_transform->position, 1.0f, camera_pos_4);
    glm_vec4_negate(camera_pos_4);
    glm_translate(view, camera_pos_4);

    // Perspective
    int width, height;
    glfwGetWindowSize(window->window, &width, &height);
    glm_perspective(glm_rad(FOV), (float)width / (float)height, NEAR, FAR,
                    projection);

    // Put everything to the respective uniforms in the shader
    ts_set_shader_uniform_mat4(mesh->shader, "model", model);
    ts_set_shader_uniform_mat4(mesh->shader, "view", view);
    ts_set_shader_uniform_mat4(mesh->shader, "projection", projection);

    // TODO: Handle the amount of vertices to draw
    glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_INT, 0);
  }
  return;
}
