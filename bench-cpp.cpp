#include <cstdio>
#include <algorithm>
#include <chrono>
#include <vector>
#include <streambuf>
#include <clientversion.h>
#include <primitives/block.h>

inline int transform_hex_symbol(int c) {
    if (c >= '0' && c <= '9') return c - '0';
    if (c >= 'a' && c <= 'f') return c - 'a' + 10;
    return -1;
}

std::vector<char> read_block_hex() {
    FILE *fp = fopen("./blocks/623200.hex", "r");
    if (!fp) {
        perror("Failed open block data");
        std::abort();
    }

    std::vector<char> data;
    while (true) {
        int sym = fgetc(fp);
        if (transform_hex_symbol(sym) == -1) {
            break;
        }

        data.push_back(sym);
    }

    fclose(fp);

    return data;
}

std::vector<char> hex_to_bytes(std::vector<char>& data) {
    std::vector<char> hex;
    hex.resize(data.size(), 0);

    for (int i = 0; i < data.size(); i += 2) {
        hex.push_back(transform_hex_symbol(data[i]) * 16 + transform_hex_symbol(data[i + 1]));
    }

    return hex;
}

class MyStreamZ {
private:
    size_t size;
    char* buf;
    size_t position;
public:
    MyStreamZ(std::vector<char>& data) : position(0) {
        size = data.size();
        buf = &*data.begin();
    }

    int GetVersion() const { return CLIENT_VERSION; }

    void read(char *pch, size_t nSize) {
        if (position + nSize > size) {
            throw std::ios_base::failure("Read attempted past buffer limit");
        }

        memcpy(pch, &buf[position], nSize);
        position += nSize;
    }

    template<typename T>
    MyStreamZ& operator>>(T&& obj) {
        // Unserialize from this stream
        ::Unserialize(*this, obj);
        return (*this);
    }
};

int main() {
    auto hex = read_block_hex();

    constexpr int iters = 100;
    int elapsed[iters];

    for (int i = 0; i < iters; ++i) {
        auto t1 = std::chrono::high_resolution_clock::now();
        auto data = hex_to_bytes(hex);
        MyStreamZ z(data);
        CBlock block;
        z >> block;
        auto t2 = std::chrono::high_resolution_clock::now();
        elapsed[i] = std::chrono::duration_cast<std::chrono::nanoseconds>(t2 - t1).count();
    }

    long long int sum = 0;
    for (int i = 0; i < iters; ++i) sum += elapsed[i];

    std::sort(elapsed, elapsed + iters);
    printf("Parse bytes (%d iterations):\n", iters);
    printf("min: %.6fms\n", elapsed[0] * 1e-6);
    printf("average: %.6fms\n", sum / iters * 1e-6);
    printf("max: %.6fms\n", elapsed[iters - 1] * 1e-6);

    return 0;
}
