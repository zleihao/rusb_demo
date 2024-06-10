use rusb::{self, constants::LIBUSB_ENDPOINT_IN, TransferType, UsbContext};
use std::time;

fn main() {
    let mut endpoint_addr = 0;
    let mut flag = false;
    let mut interface_number = 0;
    let mut device_vid = Default::default();
    let mut device_pid = Default::default();

    let context = rusb::Context::new().unwrap();

    for device in context.devices().unwrap().iter() {
        let device_desc = device.config_descriptor(0).unwrap();

        for interface in device_desc.interfaces().next().unwrap().descriptors() {
            interface_number = interface.interface_number();
            if (interface.class_code() != 3) || (interface.protocol_code() != 2) {
                continue;
            } else {
                println!("找到了鼠标");
                //遍历 Endpoint
                for endpoint in interface.endpoint_descriptors() {
                    //找到 中断传输、地址
                    if (endpoint.address() & 0x80 == LIBUSB_ENDPOINT_IN)
                        || (endpoint.transfer_type() == TransferType::Interrupt)
                    {
                        println!("addr = {:x}", endpoint.address());
                        endpoint_addr = endpoint.address();
                        device_pid = device.device_descriptor().unwrap().product_id();
                        device_vid = device.device_descriptor().unwrap().vendor_id();

                        flag = true;
                        break;
                    }
                }
            }
            if flag {
                break;
            }
        }
        if flag {
            break;
        }
    }

    let usb_handle = rusb::open_device_with_vid_pid(device_vid, device_pid).unwrap();

    //设置自动分离内核驱动
    match usb_handle.set_auto_detach_kernel_driver(true) {
        Ok(_) => println!("Successfully set auto detach kernel driver."),
        Err(e) => {
            eprintln!("Failed to set auto detach kernel driver: {:?}", e);
            return;
        }
    }

    //声明接口
    match usb_handle.claim_interface(interface_number) {
        Ok(_) => println!("Successfully claimed interface {}.", interface_number),
        Err(e) => {
            eprintln!("Failed to claim interface {}: {:?}", interface_number, e);
            return;
        }
    }

    let mut buf = vec![0u8; 8];
    let mut count = 0;
    loop {
        match usb_handle.read_interrupt(endpoint_addr, &mut buf, time::Duration::from_millis(90000))
        {
            Ok(transferred) => {
                // 成功传输，解析数据
                print!("{:04} datas: ", count);
                for &byte in buf.iter().take(transferred) {
                    print!("{:02x} ", byte);
                }
                println!();
                count += 1;
            }
            Err(rusb::Error::Timeout) => {
                // 传输超时
                eprintln!("libusb_interrupt_transfer timeout");
            }
            Err(e) => {
                // 其他传输错误
                eprintln!("libusb_interrupt_transfer error: {:?}", e);
                //break;
            }
        }
        let _ = time::Duration::from_micros(1);
    }

    let _ = usb_handle.release_interface(interface_number).unwrap();
}
