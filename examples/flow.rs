use console::{Style, Term};
use idkit::session::{AppId, BridgeUrl, Status, VerificationLevel};
use indicatif::ProgressBar;
use qrcode::{render::unicode, QrCode};
use std::{str::FromStr, time::Duration};
use tokio::time::sleep;

#[tokio::main]
async fn main() {
	let term = Term::stdout();
	term.clear_screen().unwrap();

	let session = idkit::Session::new(
		AppId::from_str("app_ce4cb73cb75fc3b73b71ffb4de178410").unwrap(),
		"test-action",
		VerificationLevel::Orb,
		BridgeUrl::default(),
		(),
		None,
	)
	.await
	.unwrap();

	let qrcode = QrCode::new(session.connect_url().to_string()).unwrap();

	term.write_line(&format!(
		"To continue, please scan the following QR code with your World App: {}",
		qrcode.render::<unicode::Dense1x2>().build(),
	))
	.unwrap();

	let pb = ProgressBar::new_spinner().with_message("Waiting for connection...");
	pb.enable_steady_tick(Duration::from_millis(100));

	let proof = loop {
		sleep(Duration::from_millis(500)).await;

		match session.poll_for_status().await.unwrap() {
			Status::WaitingForConnection => continue,
			Status::AwaitingConfirmation => {
				if pb.message() != "Waiting for confirmation..." {
					term.clear_screen().unwrap();
					pb.set_message("Waiting for confirmation...");
				}
				continue;
			},
			Status::Failed(error) => {
				term.clear_screen().unwrap();
				term.write_line("\n").unwrap();
				pb.abandon_with_message(error.to_string());
				std::process::exit(1);
			},
			Status::Confirmed(proof) => {
				pb.finish_with_message("Received proof!");
				break proof;
			},
		}
	};

	let header_style = Style::new().bold().underlined();

	term.write_line("\n").unwrap();
	term.write_line(&format!(
		"{} {:?}",
		header_style.apply_to("Credential Type:"),
		proof.credential_type,
	))
	.unwrap();

	term.write_line(&format!(
		"{} {}",
		header_style.apply_to("Nullifier Hash:"),
		proof.nullifier_hash
	))
	.unwrap();

	term.write_line(&format!(
		"{} {}",
		header_style.apply_to("Merkle Root:"),
		proof.merkle_root
	))
	.unwrap();

	term.write_line(&format!(
		"{} {}",
		header_style.apply_to("Proof:"),
		proof.proof
	))
	.unwrap();
}
