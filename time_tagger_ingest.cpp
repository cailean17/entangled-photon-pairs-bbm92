#include <TimeTagger.h>
#include <measurements/TimeTagStream.h>
#include <measurements/TimeTagStreamBuffer.h>
#include <cstdint>
#include <cstdlib>
#include <cctype>
#include <iostream>
#include <cstdio>
#include <random>
#include <vector>
#include <memory>
#include <tuple>

using namespace std;
// Ensures Event struct is not packed with alignment bits
#pragma pack(push, 1)
struct Event
{
  channel_t ch;
  timestamp_t time_ps;
};
#pragma pack(pop)

static void write_to_output(const void *ptr_to_buffer, size_t buffer_size)
{
  const uint8_t *byte_ptr_in_buffer = static_cast<const uint8_t *>(ptr_to_buffer);
  while (buffer_size)
  {
    size_t bytes_written = fwrite(byte_ptr_in_buffer, 1, buffer_size, stdout);
    if (!bytes_written)
      std::abort();
    byte_ptr_in_buffer += bytes_written;
    buffer_size -= bytes_written;
  }
}

#ifdef _WIN32
  #include <io.h>
  #include <fcntl.h>
#endif
int main(int argc, char **argv)
{
#ifdef _WIN32
  _setmode(_fileno(stdout), _O_BINARY);
#endif
  const int EVENT_OUTPUT_STREAM_SIZE = 8192;
  vector<channel_t> input_channels = {};

  for(int i = 1; i < argc;i++){
    if (isdigit(*argv[i])){
      input_channels.push_back(atoi(argv[i]));
    }
  }

  cerr << "INPUT CHANNELS: " << input_channels.size() << endl;
  // cerr << "CHECKING FOR TIME TAGGERS" << endl;
  auto devices = scanTimeTagger();
  bool use_mock = devices.empty();

  if (use_mock)
  {
    // cerr << "NO CONNECTED TIME TAGGERS, using MOCK" << endl;
  }

  unique_ptr<TimeTagStream> stream;
  if (!use_mock)
  {
    // cerr << "CREATING TIME TAGGER OBJECT: " << endl;
    auto *time_tagger = createTimeTagger();
    // cerr << "CREATED TIME TAGGER OBJECT: " << endl;
    if (!time_tagger)
      return 2;

    time_tagger->setTriggerLevel(input_channels[0], 1.5);
    time_tagger->setTriggerLevel(input_channels[1], 2.0);

    stream = make_unique<TimeTagStream>(time_tagger, EVENT_OUTPUT_STREAM_SIZE, input_channels);
  }

  vector<timestamp_t> timestamps;
  vector<channel_t> channels;
  vector<unsigned char> event_types;

  std::srand(static_cast<unsigned int>(std::time(nullptr)));
  for (;;)
  {
    if (use_mock)
    {
      // Mock buffer with 4096 events
      const size_t n_events = 4096;
      timestamps.resize(n_events);
      channels.resize(n_events);
      event_types.resize(n_events);

      // Poisonnian photon count distribution to represent SPAD-Time Tagger Output
      std::vector<Event> output;
      output.reserve(n_events);
      std::mt19937_64 gen(std::random_device{}());
      std::uniform_real_distribution<double> dis(1e-12, 1.0); // avoid 0

      double lambda_hz = 1000.0; // rate in Hz
      timestamp_t prev_time = 0;

      for (size_t i = 0; i < n_events; ++i)
      {
        double u = dis(gen);
        double dt_sec = -(std::log(u) / lambda_hz); // exponential inter-arrival
        timestamp_t dt_ps = static_cast<uint64_t>((dt_sec*1e12)); // picoseconds
        prev_time += dt_ps;

        Event ev;
        ev.ch = input_channels[i % 2]; // can randomize if needed
        ev.time_ps = prev_time;
        output.push_back(ev);
      }

      uint32_t output_size = static_cast<uint32_t>(n_events);
      write_to_output(&output_size, sizeof(output_size));
      write_to_output(output.data(), output_size * sizeof(Event));
      output.clear();
      timestamps.clear();
      channels.clear(); 
      event_types.clear();
    }
    else
    {
      // Real buffer from TimeTagger
      TimeTagStreamBuffer buf = stream->getData();
      if (buf.size == 0)
        continue;

      timestamps.resize(buf.size);
      channels.resize(buf.size);
      event_types.resize(buf.size);

      buf.getTimestamps([&timestamps](size_t)
                        { return timestamps.data(); });
      buf.getChannels([&channels](size_t)
                      { return channels.data(); });
      buf.getEventTypes([&event_types](size_t)
                        { return event_types.data(); });

      vector<Event> output;
      for (size_t i = 0; i < buf.size; i++)
      {
        // Only Take Time Tag Events
        if (static_cast<Tag::Type>(event_types[i]) != Tag::Type::TimeTag) {
          continue;
        }
        Event ev{channels[i], timestamps[i]};
        output.push_back(ev);
      }

      uint32_t output_size = static_cast<uint32_t>(output.size());
      write_to_output(&output_size, sizeof(output_size));
      write_to_output(output.data(), output_size * sizeof(Event));
    }
  }
}
