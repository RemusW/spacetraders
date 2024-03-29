use spacedust::{apis::{systems_api::*, fleet_api::{*, self}, contracts_api::{get_contracts, self}, factions_api::get_faction}, models::*};
use spacedust::apis::configuration::Configuration;
use spacedust::models::waypoint_trait::Symbol;
use spacedust::models::RefuelShipRequest;

use std::fmt::Debug;
use serde::Serialize;

pub async fn get_my_ships(conf: &Configuration) -> Option<Ship> {
    let my_ships_req = fleet_api::get_my_ships(&conf, Some(1), Some(10)).await;
    match my_ships_req {
        Ok(res) => {
            Some(res.data[0].clone())
        }
        Err(err_res) => {
            println!("{:#?}", err_res);
            None
        }
    }
}

pub async fn find_shipyward_waypoint(conf: &Configuration) -> String {
    let system_symbol_req = get_system_waypoints(&conf, "X1-HQ18", Some(1), Some(20)).await;
    let mut shipyard_waypoint: String = String::new();
    match system_symbol_req {
        Ok(res) => {
            for waypoint in res.data {
                for waypoint_trait in waypoint.traits {
                    if waypoint_trait.symbol == Symbol::Shipyard {
                        shipyard_waypoint = waypoint.symbol.to_string();
                    }
                }
            }
        }
        Err(err_res) => {
            panic!("{:#?}", err_res);
        }
    }
    shipyard_waypoint
}

// pub async fn buy_ship(conf: &Configuration, ) {
//     if shipyard_waypoint.len() == 14 {
//         let shipyard_req = get_shipyard(&conf, &shipyard_waypoint[0..7], &shipyard_waypoint).await;
//         print_req(&shipyard_req); 
//         let purchase_ship_request = PurchaseShipRequest::new(ShipType::MiningDrone, shipyard_waypoint);
//         let purchase_req = purchase_ship(&conf, Some(purchase_ship_request)).await;
//         print_req(&purchase_req);
//     }
// }

pub async fn nav_to_asteroid(conf: &Configuration, ship: &Ship) {
    let system_symbol_req = get_system_waypoints(&conf, &ship.nav.system_symbol, Some(1), Some(20)).await;
    let mut mining_waypoint: String = "".to_string();
    match system_symbol_req {
        Ok(res) => {
            // println!("{:#?}", res);
            for waypoint in res.data {
                for waypoint_trait in waypoint.traits {
                    if waypoint_trait.symbol == Symbol::MineralDeposits {
                        mining_waypoint = waypoint.symbol.to_string();
                    }
                }
            }
        }
        Err(err_res) => {
            panic!("{:#?}", err_res);
        }
    }

    if ship.nav.status == ShipNavStatus::Docked {
        let req = orbit_ship(&conf, &ship.symbol).await;
        print_req(&req);
    }

    if ship.nav.waypoint_symbol != mining_waypoint {
        let nav_req = NavigateShipRequest::new(mining_waypoint);
        let nav_ship_req = navigate_ship(&conf, &ship.symbol, Some(nav_req)).await;
        print_req(&nav_ship_req);
    }
}

pub async fn dock_and_refuel(conf: &Configuration, ship: &Ship) {
    let dock_req = dock_ship(&conf, &ship.symbol).await;
    print_req(&dock_req);
    let rr = RefuelShipRequest{
        units: Some(10),
    };
    // if let Ok(req) = get_waypoint(conf, &ship.nav.system_symbol, &ship.nav.waypoint_symbol).await {
    //     req.data.traits.contains(WaypointTrait);
    // }
    let refuel_req = refuel_ship(&conf, &ship.symbol, Some(rr)).await;
    print_req(&refuel_req);
}

pub async fn deliver_contract(conf: &Configuration, ship: &Ship) -> Result<(), Box<dyn std::error::Error>>{
    let get_contracts_resp = contracts_api::get_contracts(conf, Some(1), Some(10)).await?;
    let contract = get_contracts_resp.data.get(0).unwrap();

    // Nav to contract location
    // let faction_symbol = contract.faction_symbol.to_string();
    // let faction_resp = get_faction(conf, &faction_symbol).await?;
    // let faction_hq = faction_resp.data.headquarters;
    // let nav_req = NavigateShipRequest::new(faction_hq);

    let deliveries = contract.terms.deliver.as_ref().unwrap();
    for delivery_contract in deliveries {
        for cargo_item in ship.cargo.inventory.iter() {
            let delivery_symbol = delivery_contract.trade_symbol.to_string();
            if delivery_symbol == cargo_item.symbol {
                let units_to_deliver = delivery_contract.units_required - delivery_contract.units_fulfilled;
                if units_to_deliver > 0 {
                    // nav to delivery location
                    let nav_req = NavigateShipRequest::new(delivery_contract.destination_symbol.to_string());
                    let nav_resp = navigate_ship(conf, &ship.symbol, Some(nav_req)).await?;
                    println!("Ship will arrive at {:?}", nav_resp.data.nav.route.arrival);
                    loop {
                        let ship_nav = get_ship_nav(conf, &ship.symbol).await?;
                        if ship.nav.status == ShipNavStatus::InTransit {
                            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                            println!("... traveling")
                        }
                        else {
                            println!("Ship has arrived at {:?}", ship_nav.data.route.destination);
                            break;
                        }
                    }

                    let deliver_req = DeliverContractRequest::new(ship.symbol.to_string(), delivery_symbol, cargo_item.units);           
                    let delivery_resp = contracts_api::deliver_contract(conf, &contract.id, Some(deliver_req)).await;
                    print_req(&delivery_resp);
                }
            }
        }
    }
    
    Ok(())
}

fn print_req<T: Debug+Serialize, E: Debug>(req: &Result<T, E>) {
    match req {
        Ok(res) => {
            // res = serde_json::to_string_pretty(req.as_ref()).expect("msntg");
            // let mut file = File::create("log.json").expect("Failed to create");
            // file.write_all(res).expect("msnothingg");
            println!("{:#?}", res);
            // Print Symbol
        }
        Err(err_res) => {
            panic!("{:#?}", err_res);
        }
    }
}