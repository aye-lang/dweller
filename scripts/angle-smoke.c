// ANGLE smoke test: an EGL display on one backend, a 16x16 pbuffer and an
// ES 3.0 context; clear to red and read one pixel back.
//
//   rev=$(cat browser/gpu/angle.rev); a=target/angle/$rev
//   clang -I$a/include scripts/angle-smoke.c -L$a/lib -lEGL -lGLESv2 \
//     -Wl,-rpath,$a/lib -o target/angle-smoke
//   target/angle-smoke [metal|vulkan|swiftshader|gl|null]
//
// Prints the GL renderer and the pixel; exits 0 when it is 255,0,0,255
// (`null` draws no pixels, so there only GL errors count).
#include <stdio.h>
#include <string.h>

#include <EGL/egl.h>
#include <EGL/eglext.h>
#include <GLES3/gl3.h>

static int fail(const char *what) {
  fprintf(stderr, "angle-smoke: %s failed (EGL error 0x%x)\n", what, eglGetError());
  return 1;
}

int main(int argc, char **argv) {
  const char *backend = argc > 1 ? argv[1] : "metal";
  EGLint type, device = EGL_PLATFORM_ANGLE_DEVICE_TYPE_HARDWARE_ANGLE;
  if (!strcmp(backend, "metal")) {
    type = EGL_PLATFORM_ANGLE_TYPE_METAL_ANGLE;
  } else if (!strcmp(backend, "vulkan")) {
    type = EGL_PLATFORM_ANGLE_TYPE_VULKAN_ANGLE;
  } else if (!strcmp(backend, "swiftshader")) {
    type = EGL_PLATFORM_ANGLE_TYPE_VULKAN_ANGLE;
    device = EGL_PLATFORM_ANGLE_DEVICE_TYPE_SWIFTSHADER_ANGLE;
  } else if (!strcmp(backend, "gl")) {
    type = EGL_PLATFORM_ANGLE_TYPE_OPENGL_ANGLE;
  } else if (!strcmp(backend, "null")) {
    type = EGL_PLATFORM_ANGLE_TYPE_NULL_ANGLE;
  } else {
    fprintf(stderr, "angle-smoke: unknown backend %s\n", backend);
    return 2;
  }

  const EGLAttrib display_attribs[] = {
      EGL_PLATFORM_ANGLE_TYPE_ANGLE, type,
      EGL_PLATFORM_ANGLE_DEVICE_TYPE_ANGLE, device,
      EGL_NONE};
  EGLDisplay dpy = eglGetPlatformDisplay(EGL_PLATFORM_ANGLE_ANGLE,
                                         (void *)EGL_DEFAULT_DISPLAY, display_attribs);
  if (dpy == EGL_NO_DISPLAY) return fail("eglGetPlatformDisplay");
  EGLint major, minor;
  if (!eglInitialize(dpy, &major, &minor)) return fail("eglInitialize");

  const EGLint config_attribs[] = {
      EGL_RED_SIZE, 8, EGL_GREEN_SIZE, 8, EGL_BLUE_SIZE, 8, EGL_ALPHA_SIZE, 8,
      EGL_SURFACE_TYPE, EGL_PBUFFER_BIT,
      EGL_RENDERABLE_TYPE, EGL_OPENGL_ES3_BIT,
      EGL_NONE};
  EGLConfig config;
  EGLint count = 0;
  if (!eglChooseConfig(dpy, config_attribs, &config, 1, &count) || count == 0)
    return fail("eglChooseConfig");

  const EGLint pbuffer_attribs[] = {EGL_WIDTH, 16, EGL_HEIGHT, 16, EGL_NONE};
  EGLSurface surface = eglCreatePbufferSurface(dpy, config, pbuffer_attribs);
  if (surface == EGL_NO_SURFACE) return fail("eglCreatePbufferSurface");

  const EGLint context_attribs[] = {EGL_CONTEXT_MAJOR_VERSION, 3,
                                    EGL_CONTEXT_MINOR_VERSION, 0, EGL_NONE};
  EGLContext ctx = eglCreateContext(dpy, config, EGL_NO_CONTEXT, context_attribs);
  if (ctx == EGL_NO_CONTEXT) return fail("eglCreateContext");
  if (!eglMakeCurrent(dpy, surface, surface, ctx)) return fail("eglMakeCurrent");

  glClearColor(1.0f, 0.0f, 0.0f, 1.0f);
  glClear(GL_COLOR_BUFFER_BIT);
  unsigned char px[4] = {0, 0, 0, 0};
  glReadPixels(8, 8, 1, 1, GL_RGBA, GL_UNSIGNED_BYTE, px);
  GLenum err = glGetError();

  printf("backend %s, EGL %d.%d\n", backend, major, minor);
  printf("GL_RENDERER %s\n", (const char *)glGetString(GL_RENDERER));
  printf("GL_VERSION %s\n", (const char *)glGetString(GL_VERSION));
  printf("pixel %d,%d,%d,%d glGetError 0x%x\n", px[0], px[1], px[2], px[3], err);

  eglMakeCurrent(dpy, EGL_NO_SURFACE, EGL_NO_SURFACE, EGL_NO_CONTEXT);
  eglDestroyContext(dpy, ctx);
  eglDestroySurface(dpy, surface);
  eglTerminate(dpy);

  int red = px[0] == 255 && px[1] == 0 && px[2] == 0 && px[3] == 255;
  if (!strcmp(backend, "null")) red = 1;
  printf("%s\n", red ? "ok" : "FAIL");
  return red && err == GL_NO_ERROR ? 0 : 1;
}
