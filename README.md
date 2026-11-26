# RK Festival · Minecraft Java Edition 26.3

Server Rust native cho Linux ARMv7 (RK3229). ZIP có mã nguồn, Cargo.lock, tài nguyên đấu trường, binary musl liên kết tĩnh và công cụ kiểm thử. Server chỉ dùng thư viện chuẩn Rust. Người chơi dùng client Java Edition 26.3 trên máy riêng. Tất cả người sẵn sàng trước giờ bắt đầu cùng vào một sân. Người đến giữa ván theo dõi từ khán đài.

## Chạy trên box

```bash
chmod +x bin/rk-festival-armv7
sha256sum -c SHA256SUMS
ip -br -4 addr
./bin/rk-festival-armv7 --listen 192.168.1.50:25565 --game all --max-players 50 --stats
```

Thay IP mẫu bằng IP của box. Dừng tiến trình cũ nếu dùng chung địa chỉ và cổng. Đây là bản offline cho LAN tin cậy. Trong Minecraft, chọn Multiplayer → Direct Connection và nhập địa chỉ đã bind. Server mặc định 50 chỗ. Ctrl+C dừng server. Vị trí, vòng chơi và điểm thắng được giữ trong RAM cho phiên kết nối.

## Chat và Đua sáu cổng

Nhấn **T**, nhập tin nhắn và nhấn **Enter**. Mọi người, kể cả khán giả, đều thấy tên và nội dung. Tiếng Việt và emoji được hỗ trợ. Server phát lại tin dưới dạng tin hệ thống. Mỗi kết nối gửi tối đa một tin mỗi nửa giây.

Để thử bản đồ ngay, chạy playlist sau với IP LAN của box.

```bash
./bin/rk-festival-armv7 --listen 192.168.1.50:25565 --game rally,king,tnt --stats
```

Bản đồ 80 × 67 block có đường chạy xuyên rừng thông, bến thuyền buồm, cầu gỗ qua hồ, tháp canh, đèo tuyết và cổng khải hoàn. Nhảy qua thân cây và hai khe trên cầu. Sidebar hiển thị số cổng đã qua, tên và tọa độ X/Z của cổng tiếp theo. Rơi xuống hồ hoặc khỏi bản đồ sẽ đưa bạn về cổng gần nhất đã qua. Hết 150 giây, người qua nhiều cổng nhất thắng nếu đã qua ít nhất một cổng. Người cùng về đích trong một tick được đồng hạng.

Người vào giữa cuộc đua xuất hiện trên ban công và có thể chat. Hết ván, mọi người trở về sảnh. Server gửi 56 chunk đầy đủ gồm cả vành ngoài, hình khối, ánh sáng và bản đồ độ cao. Hai phiên bản sân được tạo sẵn khi biên dịch.

## Chọn trò

```bash
./bin/rk-festival-armv7 --list
./bin/rk-festival-armv7 --listen 192.168.1.50:25565 --game snow,pipe,paint --stats
```

Danh sách có thứ tự. Sau trò cuối, server quay lại trò đầu. Một mã duy nhất sẽ lặp trò đó. Sảnh chờ 8 giây, title đếm ngược trong 3 giây cuối. Đua sáu cổng có 150 giây. Các trò khác có 60 giây. Trò loại trực tiếp có thể kết thúc sớm. Ở trò tính điểm, nhiều người đứng đầu có thể đồng thắng nếu đã ghi điểm. Sidebar cập nhật mục tiêu, tình trạng trận, điểm cá nhân hoặc đội và số ván thắng trong phiên. Title + subtitle giữa màn hình báo hiệu chuyển trò, mục tiêu, bị loại và kết quả.

| Mã | Luật và cách thắng |
|---|---|
| rally | Theo đường đánh dấu vàng và qua 6 cổng theo thứ tự. Người đầu tiên đến đích thắng. |
| color | Chạy đến màu được gọi. Hết hạn, ô sai biến mất. |
| memory | Ghi nhớ vị trí các màu. Khi sàn trắng, chạy đến vị trí màu được gọi. |
| wave | Tìm khe tránh vạch đỏ quét ngang hoặc dọc. Mỗi người bắt đầu với 3 tim. |
| tnt | Bước lên TNT khiến ô sàn rơi sau 0,4 giây. Có hai tầng và một sân chung. |
| paint | Chạy để sơn sàn. Đội sở hữu nhiều ô nhất khi hết giờ thắng. |
| beacon | Chạm ô vàng lấy điểm. Đốm sáng đổi chỗ khi có người chạm hoặc sau 5 giây. |
| traffic | Chạy về vạch vàng khi xanh. Di chuyển lúc đỏ đưa bạn về đầu đường. Mỗi lần về đích cộng 1 điểm. |
| treasure | Cùng lật ô để tìm đủ 5 chìa khóa trong 60 giây. Ô xanh lá báo gần một vị trí giấu khóa. |
| king | Đứng trên vùng xanh để lấy điểm mỗi giây. Vùng di chuyển sau 7 giây. |
| ice | Băng chuyển đỏ trước khi vỡ. Ai trụ lại lâu nhất thắng. |
| islands | Chạy lên đảo xanh trước khi các ô nước biến mất. Đảo nhỏ dần qua từng lượt. |
| mine | Nhớ vị trí mìn đỏ rồi đứng lên ô an toàn khi toàn bộ sàn bị che. |
| snow | Trong nhóm người tuyết bằng block, tìm người có đầu bí ngô rồi chạy đến gần. Mỗi lần tìm được cộng 1 điểm. |
| pipe | Chạy tới ống gạch có đầu quái xanh, nhìn thẳng mục tiêu và nhấp chuột trong tầm 4,5 block để lấy điểm. |

Các mô hình mùa đông dùng block vanilla (tuyết, bí ngô, gạch và đầu creeper). Không cần resource pack. Sân 18 × 18 tái sử dụng cùng dữ liệu chunk/registry 26.3 từ bản RK Party, có thêm các thay đổi block theo trò. Server hiển thị tiếng Việt.

## Test trên laptop Linux Mint

```bash
sudo apt install qemu-user
qemu-arm -cpu cortex-a7 ./bin/rk-festival-armv7 --listen 127.0.0.1:25565 --game color,paint --stats
```

Ở terminal khác, từ thư mục này và lúc server trống, chạy:

```bash
python3 tools/check_party.py --players 50
```

Bài test cần playlist bắt đầu bằng color,paint và cổng mặc định (có thể truyền --port). Mô phỏng client thật ở cấp giao thức: đăng nhập, nhận chunk, nhìn người cùng sân, nhận title, subtitle, sidebar, chọn ô đúng/sai, nhận kết quả và sang trò tiếp. Có thể mở Minecraft đồ họa kết nối localhost trong lượt test riêng. Số đo TPS trên giả lập không đại diện cho RK3229 vật lý.

## Build lại trên Linux x86-64

Cần rustup; bản phát hành được build với Rust 1.89.0. Linker rust-lld được cấu hình trong .cargo/config.toml.

```bash
rustup target add armv7-unknown-linux-musleabihf
cargo test --locked
cargo build --locked --release -j 2 --target armv7-unknown-linux-musleabihf
```

Binary mới ở target/armv7-unknown-linux-musleabihf/release/rk-festival. Để chạy unit test trực tiếp trên mã ARM bằng QEMU:

```bash
CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_RUNNER="qemu-arm -cpu cortex-a7" cargo test --locked --target armv7-unknown-linux-musleabihf
```

src/net.rs xử lý login và kết nối, src/wire.rs đọc ghi gói giao thức, src/arena.rs quản lý đấu trường chung và HUD, src/games.rs triển khai 15 luật chơi. src/game_tests.rs chứa các trường hợp kiểm tra điểm, phân đội, xét loại và kết quả.

Để kiểm tra chat và đường đua, khởi động server với `--game rally,paint` rồi chạy `python3 tools/check_rally.py` ở terminal khác. Bài test dùng hai tay đua và một khán giả.
