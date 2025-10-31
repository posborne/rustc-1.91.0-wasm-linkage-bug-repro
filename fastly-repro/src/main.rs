fn main() {
    unsafe {
        let mut req = fastly_shared::INVALID_REQUEST_HANDLE;
        let mut body = fastly_shared::INVALID_BODY_HANDLE;
        fastly_sys::fastly_http_req::body_downstream_get(&mut req, &mut body)
            .result()
            .unwrap();
        let mut new_body = fastly_shared::INVALID_BODY_HANDLE;
        fastly_sys::fastly_http_body::new(&mut new_body)
            .result()
            .unwrap();
        let mut resp = fastly_shared::INVALID_RESPONSE_HANDLE;
        fastly_sys::fastly_http_resp::new(&mut resp)
            .result()
            .unwrap();
        fastly_sys::fastly_http_resp::status_set(resp, 200)
            .result()
            .unwrap();
        let streaming = 0;
        fastly_sys::fastly_http_resp::send_downstream(resp, new_body, streaming)
            .result()
            .unwrap();
        fastly_sys::fastly_http_req::close(req).result().unwrap();
        fastly_sys::fastly_http_body::close(body).result().unwrap();
    }
}