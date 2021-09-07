  - kubeadm init --cri-socket /run/containerd/containerd.sock --token ${kubeadm_token}
  - mkdir -p /home/opensuse/.kube
  - sudo cp -i /etc/kubernetes/admin.conf /home/opensuse/.kube/config
  - sudo chown opensuse /home/opensuse/.kube/config
  - export KUBECONFIG=/etc/kubernetes/admin.conf
%{ if workers == "0" }
  - kubectl taint nodes $(kubectl get nodes --selector=node-role.kubernetes.io/master | awk 'FNR==2{print $1}') node-role.kubernetes.io/master-
%{ endif }
  - cilium install
  - helm repo add jetstack https://charts.jetstack.io
  - helm repo add kubewarden https://charts.kubewarden.io
  - helm repo update
  - kubectl apply -f https://github.com/jetstack/cert-manager/releases/download/v1.6.0/cert-manager.yaml
  - kubectl wait --for=condition=Available deployment --timeout=2m -n cert-manager --all
  - helm install -n kube-system kubewarden-crds kubewarden/kubewarden-crds
  - helm install --wait -n kube-system kubewarden-controller kubewarden/kubewarden-controller
  - kubectl apply -f /usr/local/src/lockc/contrib/kubernetes/lockc.yaml
  - kubectl wait --for=condition=Available daemonset --timeout=2m -n lockcd --all
  - kubectl apply -f /usr/local/src/lockc/contrib/kubernetes/kubewarden.yaml
