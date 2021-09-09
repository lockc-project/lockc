  - kubeadm init --cri-socket /run/containerd/containerd.sock
  - mkdir -p /home/opensuse/.kube
  - sudo cp -i /etc/kubernetes/admin.conf /home/opensuse/.kube/config
  - sudo chown opensuse /home/opensuse/.kube/config
%{ if workers == "0" }
  - export KUBECONFIG=/etc/kubernetes/admin.conf
  - kubectl taint nodes $(kubectl get nodes --selector=node-role.kubernetes.io/master | awk 'FNR==2{print $1}') node-role.kubernetes.io/master-
%{ endif }
