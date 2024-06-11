/*
 * Isabelle project
 *
 * Copyright 2023-2024 Maxim Menshikov
 *
 * Permission is hereby granted, free of charge, to any person obtaining
 * a copy of this software and associated documentation files (the “Software”),
 * to deal in the Software without restriction, including without limitation
 * the rights to use, copy, modify, merge, publish, distribute, sublicense,
 * and/or sell copies of the Software, and to permit persons to whom the
 * Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included
 * in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS
 * OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 * FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 * DEALINGS IN THE SOFTWARE.
 */
use actix_web::web;
use isabelle_dm::data_model::item::Item;
use isabelle_dm::data_model::process_result::ProcessResult;
use isabelle_plugin_api::api::*;

use log::info;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

fn unset_id() -> u64 {
    return u64::MAX;
}

struct SamplePlugin {}

impl Plugin for SamplePlugin {
    fn ping_test(&mut self) {}

    fn item_pre_edit_hook(
        &mut self,
        _api: &Box<dyn PluginApi>,
        _hndl: &str,
        _user: &Option<Item>,
        _collection: &str,
        _old_itm: Option<Item>,
        _itm: &mut Item,
        _del: bool,
        _merge: bool,
    ) -> ProcessResult {
        return ProcessResult {
            succeeded: false,
            error: "not implemented".to_string(),
        };
    }

    fn item_post_edit_hook(
        &mut self,
        api: &Box<dyn PluginApi>,
        hndl: &str,
        collection: &str,
        id: u64,
        del: bool,
    ) {
        if hndl != "sampleplugin_post_edit" || collection != "config" || !del {
            return;
        }

        /* try to take edited item from database */
        let item_opt = api.db_get_item(collection, id);
        if item_opt.is_none() {
            info!("No item");
            return;
        }

        /* unwrap item and do things with it */
        let mut item = item_opt.unwrap().clone();

        /* here, just swap values */
        let old_val = item.safe_str("xml", "");
        let new_val;
        if old_val == "" || old_val == "old" {
            new_val = "new";
        } else {
            new_val = "old";
        }
        item.set_str("xml", new_val);

        /* save to database with new 'xml' field */
        api.db_set_item(collection, &item, false /* this is complete item, we don't merge it in */);
    }

    fn item_auth_hook(
        &mut self,
        _api: &Box<dyn PluginApi>,
        _hndl: &str,
        _: &Option<Item>,
        _: &str,
        _: u64,
        _: Option<Item>,
        _: bool,
    ) -> bool {
        return true;
    }

    fn item_list_filter_hook(
        &mut self,
        _api: &Box<dyn PluginApi>,
        _hndl: &str,
        _user: &Option<Item>,
        _collection: &str,
        _context: &str,
        _map: &mut HashMap<u64, Item>,
    ) {
    }

    fn route_url_hook(
        &mut self,
        _api: &Box<dyn PluginApi>,
        _hndl: &str,
        _: &Option<Item>,
        _: &str,
    ) -> WebResponse {
        return WebResponse::NotImplemented;
    }

    fn route_url_post_hook(
        &mut self,
        api: &Box<dyn PluginApi>,
        hndl: &str,
        user: &Option<Item>,
        query: &str,
        post_itm: &Item,
    ) -> WebResponse {
        if hndl != "sampleplugin_import" {
            return WebResponse::NotImplemented;
        }

        if !api.auth_check_role(&user, "admin") {
            return WebResponse::Unauthorized;
        }

        /* those are possible parameters */
        #[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
        struct ImportParams {
            #[serde(default = "unset_id")]
            pub id: u64,
        }

        /* unroll parameters */
        let params = web::Query::<ImportParams>::from_query(query).unwrap();
        let input_xml = post_itm.safe_str("xml", "");

        /* process xml and make item out of it */
        let mut itm = Item::new();
        itm.id = params.id;
        itm.set_str("xml", &input_xml);

        /* save new (or old?) item to database */
        api.db_set_item("config", &itm, false /* not merging */);

        return WebResponse::Ok;
    }

    fn route_unprotected_url_hook(
        &mut self,
        _api: &Box<dyn PluginApi>,
        _hndl: &str,
        _: &Option<Item>,
        _: &str,
    ) -> WebResponse {
        return WebResponse::NotImplemented;
    }

    fn route_unprotected_url_post_hook(
        &mut self,
        _api: &Box<dyn PluginApi>,
        _hndl: &str,
        _: &Option<Item>,
        _: &str,
        _: &Item,
    ) -> WebResponse {
        return WebResponse::NotImplemented;
    }

    fn collection_read_hook(
        &mut self,
        _api: &Box<dyn PluginApi>,
        _hndl: &str,
        _collection: &str,
        _itm: &mut Item,
    ) -> bool {
        return false;
    }

    fn call_otp_hook(&mut self, _api: &Box<dyn PluginApi>, _hndl: &str, _itm: &Item) {
    }
}

#[no_mangle]
pub fn register(api: &mut dyn PluginPoolApi) {
    api.register(Box::new(SamplePlugin {}));
}
