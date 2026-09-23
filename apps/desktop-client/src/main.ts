import { startApp } from "./app";
import { startOverlayApp } from "./overlay";

if (location.hash === "#overlay") {
  void startOverlayApp();
} else {
  void startApp();
}
