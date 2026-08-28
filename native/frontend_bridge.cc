#include <algorithm>
#include <cstddef>
#include <cstdint>
#include <new>

extern "C" {
#include "tensorflow/lite/experimental/microfrontend/lib/frontend.h"
#include "tensorflow/lite/experimental/microfrontend/lib/frontend_util.h"
}
namespace {
constexpr int kSampleRate = 16000;
constexpr int kWindowMs = 30;
constexpr int kStepMs = 10;
constexpr int kFeatureCount = 40;

struct MwwFrontend {
  FrontendConfig config{};
  FrontendState state{};
};

void Configure(FrontendConfig* config) {
  config->window.size_ms = kWindowMs;
  config->window.step_size_ms = kStepMs;

  config->filterbank.num_channels = kFeatureCount;
  config->filterbank.lower_band_limit = 125.0f;
  config->filterbank.upper_band_limit = 7500.0f;

  config->noise_reduction.smoothing_bits = 10;
  config->noise_reduction.even_smoothing = 0.025f;
  config->noise_reduction.odd_smoothing = 0.06f;
  config->noise_reduction.min_signal_remaining = 0.05f;

  config->pcan_gain_control.enable_pcan = 1;
  config->pcan_gain_control.strength = 0.95f;
  config->pcan_gain_control.offset = 80.0f;
  config->pcan_gain_control.gain_bits = 21;

  config->log_scale.enable_log = 1;
  config->log_scale.scale_shift = 6;
}
}  // namespace

extern "C" MwwFrontend* mww_frontend_create() {
  auto* frontend = new (std::nothrow) MwwFrontend();
  if (frontend == nullptr) {
    return nullptr;
  }
  Configure(&frontend->config);
  if (!FrontendPopulateState(&frontend->config, &frontend->state, kSampleRate)) {
    delete frontend;
    return nullptr;
  }
  return frontend;
}

extern "C" void mww_frontend_destroy(MwwFrontend* frontend) {
  if (frontend != nullptr) {
    FrontendFreeStateContents(&frontend->state);
    delete frontend;
  }
}

extern "C" int mww_frontend_process(MwwFrontend* frontend,
                                    const int16_t* samples,
                                    size_t sample_count,
                                    uint16_t* output,
                                    size_t output_capacity) {
  if (frontend == nullptr || samples == nullptr || output == nullptr) {
    return -1;
  }

  size_t samples_read = 0;
  const FrontendOutput result = FrontendProcessSamples(
      &frontend->state, samples, sample_count, &samples_read);
  if (result.size == 0) {
    return 0;
  }
  if (result.size > output_capacity) {
    return -2;
  }

  std::copy_n(result.values, result.size, output);
  return static_cast<int>(result.size);
}
